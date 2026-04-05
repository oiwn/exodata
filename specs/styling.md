# Current Context

## Component Styling Cleanup

### Problem

- Some frontend components, especially `stellarhost_detail`, now carry large amounts of inline Tailwind utility classes.
- This makes Rust view code harder to scan and mixes structure, styling, and decorative art-direction too tightly.
- The current issue is not functionality. It is readability and maintainability of existing component code.

### Goal

- Refactor existing components to reduce very large inline class strings.
- Keep component structure easier to read.
- Preserve the current visual design while choosing a cleaner styling pattern.
- Avoid unnecessary project-wide styling churn.

### Possible Approaches

#### 1. Tailwind + Semantic Classes

- Keep Tailwind in the project.
- Move large section-level styling into semantic class names such as:
  - `.host-hero`
  - `.host-hero__title`
  - `.host-provenance`
  - `.star-visual`
- Define those classes in CSS/SCSS files and use `@apply` where helpful.

Pros:

- lowest disruption
- works with the current stack
- keeps utility classes available for simple layout/text tweaks

Cons:

- classes remain global unless naming stays disciplined

#### 2. Feature CSS Files Imported Into Main Stylesheet

- Keep styling global, but split it into dedicated files such as:
  - `style/components/stellarhost-detail.css`
- Import those files into the main stylesheet bundle.

Pros:

- simple mental model
- styles are easier to locate by feature
- no new styling library required

Cons:

- still global CSS

#### 3. Scoped Component Stylesheets

- Use a scoped styling approach such as Stylance.
- Keep stylesheet files next to components, for example:
  - `hero.rs`
  - `hero.module.scss`

Pros:

- strongest component ownership
- better isolation for complex feature UI
- good fit for CSS-heavy sections

Cons:

- introduces extra styling tooling and conventions

### Current Direction

- No final choice yet.
- The next refactor pass should compare:
  - Tailwind + semantic classes
  - feature CSS files imported into the main stylesheet
  - scoped component styles for `stellarhost_detail`
- Prefer the option that improves readability without adding unnecessary styling complexity.
