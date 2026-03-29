# Module Augmentation — Declaration Merging, Third-Party Types, Global Scope

## When to Use Module Augmentation

- Adding properties to third-party types (Express `Request`, React `JSX.IntrinsicElements`)
- Extending global types (`Window`, `Array`, `NodeJS.ProcessEnv`)
- Adding custom fields to framework-specific interfaces

---

## Augmenting a Third-Party Module

The file **must** contain at least one `import` or `export` to be treated as a module.
Without this, TypeScript treats it as an ambient script and the augmentation silently
fails or applies globally.

### Express example

```typescript
// types/express.d.ts
import "express";

declare module "express-serve-static-core" {
  interface Request {
    currentUser?: { id: string; role: "admin" | "guest" };
    requestId: string;
  }
}
```

### React JSX augmentation

```typescript
// types/react.d.ts
import "react";

declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      "my-web-component": React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & { theme?: string },
        HTMLElement
      >;
    }
  }
}
```

### Environment variables

```typescript
// types/env.d.ts
declare namespace NodeJS {
  interface ProcessEnv {
    NODE_ENV: "development" | "production" | "test";
    DATABASE_URL: string;
    API_KEY: string;
  }
}
```

Note: `ProcessEnv` augmentation is on a namespace, not a module — no import/export
needed for ambient namespace declarations.

---

## Augmenting Global Scope

Use `declare global` inside a module file (one with imports/exports):

```typescript
// lib/extensions.ts
export {}; // forces module context

declare global {
  interface Array<T> {
    last(): T | undefined;
  }

  interface Window {
    analytics: { track: (event: string, data?: object) => void };
  }

  // Add a global variable
  var __APP_VERSION__: string;
}
```

---

## Interface Merging Rules

TypeScript merges multiple `interface` declarations with the same name in the same scope:

```typescript
interface Config {
  host: string;
}
interface Config {
  port: number;
}
// Result: { host: string; port: number }
```

### Merge limitations

- You **cannot change** the type of an existing property — both declarations must agree
- You **cannot add new top-level declarations** in an augmentation — only extend existing
  interfaces/namespaces
- **Default exports cannot be augmented** — only named exports
- Later declarations take **higher priority** for overloaded function signatures
- `type` aliases do **not** merge — duplicate `type` names cause an error

---

## Common Mistakes

### Missing module context

```typescript
// WRONG — no import/export, treated as ambient script
declare module "express" {
  interface Request {
    user?: User;
  }
}
// This creates a NEW ambient module declaration, not an augmentation

// CORRECT — import makes it a module, augmentation works
import "express";
declare module "express-serve-static-core" {
  interface Request {
    user?: User;
  }
}
```

### Wrong module name

Express types live in `express-serve-static-core`, not `express`. Always check the
`.d.ts` files of the package to find where the interface is actually declared:

```bash
# Find where Request is declared
grep -r "interface Request" node_modules/@types/express*/
```

### Conflicting .d.ts and .ts files

If a `.d.ts` file has the same name as a `.ts` file in the same directory, the `.d.ts`
is **silently ignored**. Use distinct file names.

### Augmenting non-exported types

You can only augment types that are exported from the module. Internal types (not in
the public API) cannot be augmented. If you need to modify an internal type, use module
patching or fork the type definitions.

---

## Namespace Merging

Namespaces can be merged with classes, functions, or enums to add static properties:

```typescript
function buildValidator(value: unknown): boolean {
  return typeof value === "string";
}

namespace buildValidator {
  export function isEmail(value: string): boolean {
    return value.includes("@");
  }
}

// Both work:
buildValidator("test"); // function call
buildValidator.isEmail("a@b"); // namespace access
```

This pattern is useful for adding utility methods to functions (like `React.memo` on a
component) but should be used sparingly — prefer modules over namespace augmentation
in modern code.

---

## Variance Annotations (`in`, `out`) — Quick Reference

Introduced in TypeScript 4.7. Declares the variance of type parameters explicitly:

```typescript
type Producer<out T> = () => T; // covariant
type Consumer<in T> = (x: T) => void; // contravariant
type Mapper<in T, out U> = (x: T) => U; // mixed
type Processor<in out T> = (x: T) => T; // invariant
```

### When to use

- Large codebases where variance computation slows type checking
- Documentation of intent for library consumers
- Recursive types where TypeScript cannot infer variance

### Critical rule

**Never write a variance annotation that contradicts the structural variance.** Wrong
annotations cause silent unsound behavior. TypeScript catches some errors but not all.
If in doubt, omit the annotation — TypeScript's inference is correct by default.
