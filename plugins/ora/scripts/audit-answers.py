#!/usr/bin/env python3
"""Audit logged ora:explore / ora:research answers for versions and dates no source carries.

Reads the jsonl the SubagentStop hook writes (default ~/.claude/ora/answers.jsonl).

The deterministic pass is the enforcer: pull every version and date out of the
conclusion and check each one appears in the evidence the same answer cites. A
token the question itself named is the asker's premise, not a finding, so it is
not counted against the answer -- an auditor that cannot see the question
measures the caller instead of the answer.

`--jev` adds a second opinion from TypeSafe's decision model over the same rows.
It is a discovery pass, never enforcement: it reports and changes no verdict.
Needs JEV_API_KEY. Roughly $0.00002 and under a second per row.
"""

import argparse
import json
import os
import pathlib
import re
import sys
import urllib.request

DEFAULT_LOG = pathlib.Path.home() / ".claude" / "ora" / "answers.jsonl"

VERSION = re.compile(r"v?\d+\.\d+(?:\.\d+)?(?:[-+][0-9A-Za-z.]+)?")
ISO_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")
LONG_DATE = re.compile(
    r"(?:January|February|March|April|May|June|July|August|September|October"
    r"|November|December)\s+\d{1,2},?\s+\d{4}"
)
MONTH_NAMES = (
    "January February March April May June July August September "
    "October November December".split()
)
MONTHS = {
    m: i
    for i, m in enumerate(
        "January February March April May June July August September "
        "October November December".split(),
        start=1,
    )
}

CONCLUSION = re.compile(
    r"<conclusion>(.*?)</conclusion>|^#{1,4}\s*Conclusion\s*$(.*?)(?=^#{1,4}\s|\Z)",
    re.DOTALL | re.IGNORECASE | re.MULTILINE,
)


def conclusion_of(answer):
    match = CONCLUSION.search(answer)
    if match:
        return (match.group(1) or match.group(2) or "").strip()
    # No conclusion block: judge the whole answer rather than skip the row.
    return answer.strip()


def evidence_of(answer, conclusion):
    """Everything in the answer that is not the conclusion -- quotes, URLs, body."""
    return answer.replace(conclusion, " ", 1) if conclusion else answer


def iso_of(long_date):
    month, day, year = re.split(r"[\s,]+", long_date.strip())
    return f"{int(year):04d}-{MONTHS[month]:02d}-{int(day):02d}"


def long_of(iso_date):
    """`2026-09-09` -> the ways prose spells it. The conversion has to run both ways:
    normalising only long->ISO flagged a conclusion dated `2026-09-09` as uncited against
    evidence that said `September 9, 2026`."""
    year, month, day = (int(part) for part in iso_date.split("-"))
    name = MONTH_NAMES[month - 1]
    return {f"{name} {day}, {year}", f"{name} {day} {year}"}


def claims_of(text):
    """Version and date tokens, each with the spellings that mean the same thing.

    Tokens keep their dots. Normalising `4.1.5` to `415` once let a commit sha
    absolve an uncited version, so the haystack and the needle stay literal.
    """
    found = {}
    for token in VERSION.findall(text):
        found.setdefault(token, {token.lstrip("vV"), "v" + token.lstrip("vV")})
    for token in ISO_DATE.findall(text):
        spellings = {token}
        try:
            spellings |= long_of(token)
        except (ValueError, IndexError):
            pass
        found.setdefault(token, spellings)
    for token in LONG_DATE.findall(text):
        spellings = {token, token.replace(",", "")}
        try:
            spellings.add(iso_of(token))
        except (ValueError, KeyError):
            pass
        found.setdefault(token, spellings)
    return found


def carried(spellings, haystack):
    return any(spelling.lower() in haystack for spelling in spellings)


def audit(row):
    """(conclusion, uncited, premises, bare).

    `bare` means the answer arrived with no evidence beside its conclusion -- the
    caller asked for the value alone and the agent dropped the results block.
    There is nothing to check a claim against, so such a row is reported as its
    own outcome rather than counted as an uncited claim.
    """
    answer = row.get("answer") or ""
    question = (row.get("question") or "").lower()
    conclusion = conclusion_of(answer)
    evidence = evidence_of(answer, conclusion)
    if not evidence.strip():
        return conclusion, [], [], True

    haystack = evidence.lower()
    uncited, premises = [], []
    for token, spellings in claims_of(conclusion).items():
        if carried(spellings, haystack):
            continue
        if carried(spellings, question):
            premises.append(token)
        else:
            uncited.append(token)
    return conclusion, uncited, premises, False


JEV_QUESTION = {
    "carried": {
        "type": "choice",
        "instructions": (
            "Every version number and every date the conclusion states must appear in the "
            "evidence the answer cites. A version or date that the question itself named is "
            "the asker's premise, not a claim, and does not count against the answer. "
            "Is every remaining version and date carried by the evidence?"
        ),
        "criteria": {
            "carried": "Every version and date the conclusion asserts appears in the cited evidence.",
            "not_carried": "The conclusion asserts a version or date that the evidence does not contain.",
        },
    }
}


# The API returns token counts, not a cost. Rate from docs.typesafe.ai/models: input only,
# $0.042 per million, output free.
USD_PER_INPUT_TOKEN = 0.042 / 1_000_000


def ask_jev(question, conclusion, evidence):
    body = json.dumps(
        {
            "model": "jev-latest",
            "state": {
                "question": question,
                "conclusion": conclusion,
                "evidence": evidence[:8000],
            },
            "questions": JEV_QUESTION,
        }
    ).encode()
    request = urllib.request.Request(
        "https://api.typesafe.ai/v1/systemone",
        data=body,
        headers={
            "Authorization": "Bearer " + os.environ["JEV_API_KEY"],
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        payload = json.load(response)
    answer = payload["answers"]["carried"]
    cost = payload["usage"]["input_tokens"] * USD_PER_INPUT_TOKEN
    return answer["choice"], answer["confidence"], cost


def rows_of(path):
    """Last row per agent_id -- SubagentStop can fire more than once per subagent."""
    latest = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            try:
                row = json.loads(line)
            except ValueError:
                continue
            latest[row.get("agent_id") or id(row)] = row
    return list(latest.values())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", nargs="?", type=pathlib.Path, default=DEFAULT_LOG)
    parser.add_argument("--jev", action="store_true", help="add a Jev second opinion")
    parser.add_argument("--agent", help="only rows from this agent, e.g. ora:research")
    args = parser.parse_args()

    if not args.log.exists():
        sys.exit(f"no answer log at {args.log}")

    rows = rows_of(args.log)
    if args.agent:
        rows = [row for row in rows if row.get("agent") == args.agent]
    if not rows:
        sys.exit("no rows to audit")

    flagged = bare = spend = 0
    for row in rows:
        conclusion, uncited, premises, no_evidence = audit(row)
        verdict = "UNCITED" if uncited else "bare" if no_evidence else "ok"
        line = f"{row.get('ts','?')}  {row.get('agent','?'):12} {verdict:8}"
        if uncited:
            flagged += 1
            line += f" {', '.join(uncited)}"
        if no_evidence:
            bare += 1
        if premises:
            line += f"  (premise: {', '.join(premises)})"

        if args.jev and not no_evidence:
            try:
                answer = row.get("answer") or ""
                choice, confidence, cost = ask_jev(
                    row.get("question") or "", conclusion, evidence_of(answer, conclusion)
                )
                spend += cost
                line += f"  | jev {choice} {confidence:.2f}"
            except Exception as error:  # a second opinion must not break the report
                line += f"  | jev unavailable ({type(error).__name__})"
        print(line)

    print(f"\n{flagged} of {len(rows)} answers assert a version or date no source carries.")
    print(f"{bare} of {len(rows)} came back with no evidence beside the conclusion.")
    if args.jev:
        print(f"jev: ${spend:.6f}")


if __name__ == "__main__":
    main()
