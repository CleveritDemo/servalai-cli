---
name: senior-designer
description: UX/Product Design heuristics for web, mobile, platform, and CLI/TUI interfaces. Accessibility, interaction design, design systems, component specs, user journeys. Load when designing or reviewing any user-facing interface or experience.
metadata:
  audience: senior-designer, architect, fullstack-lt, developer
---

# Senior Designer

## When to Use

- Designing a new UI, screen, or user flow
- Reviewing front-end code for UX quality or accessibility
- Selecting or extending a design system
- Specifying component states and interaction patterns
- Conducting an accessibility audit (WCAG)
- Designing AI interaction patterns (chat, suggestion UX, explainability)
- Designing data visualisation or analytics dashboards
- Designing CLI/TUI user experiences (help text, error messages, output formatting)

## Triggers

ux, ui, design, interface, user experience, accessibility, wcag, component, typography, colour, layout, interaction, flow, journey, wireframe, figma, design system, mobile, responsive, dashboard, visualisation, dark mode, form, error state, empty state, onboarding

## Core Workflow

1. **Define the user** — persona, goal, context (device, environment, technical level).
2. **Audit current state** — existing UI, design system, component library. Cite with `file_path:line_number`.
3. **Identify the real problem** — distinguish symptom from root cause UX failure.
4. **Apply design principles** (see below).
5. **Specify interaction** — states, transitions, copy, accessibility requirements.
6. **Define handoff artefacts** — component structure, tokens, responsive rules, ARIA.
7. **Validate accessibility** — WCAG 2.1 AA minimum.

## Design Principles

| Principle | Application |
|---|---|
| **Clarity over cleverness** | If you need to explain it, redesign it |
| **Progressive disclosure** | Show what matters now; reveal complexity on demand |
| **Feedback for every action** | Loading, success, error — always tell the user what happened |
| **Forgiveness** | Undo, confirmation for destructive actions, graceful error recovery |
| **Consistency** | Same affordance, same behaviour — always. Surprise is a bug |
| **Accessibility as default** | Design for disability; it improves the experience for everyone |
| **Performance is UX** | Perceived speed > actual speed; skeleton screens, optimistic updates |
| **Delight in the details** | Micro-interactions, thoughtful copy, satisfying feedback |

## Design Systems Reference

| System | When |
|---|---|
| **shadcn/ui + Tailwind** | Modern web apps, full control, component customisation |
| **Radix UI primitives** | Accessible headless components; pair with any styling |
| **Material Design 3** | Android native, cross-platform consistency with Google ecosystem |
| **Apple HIG** | iOS/macOS native; follow platform conventions strictly |
| **Fluent 2 (Microsoft)** | Enterprise web, Microsoft ecosystem |
| **Ant Design** | Data-dense enterprise dashboards |
| **Custom design tokens** | Always — regardless of component library, tokens are the source of truth |

## Accessibility Checklist (WCAG 2.1 AA)

### Visual
- [ ] Colour contrast ≥ 4.5:1 (normal text), ≥ 3:1 (large text / UI components)
- [ ] Information not conveyed by colour alone
- [ ] Text resizable to 200% without loss of content or functionality
- [ ] No flashing content > 3Hz

### Interactive
- [ ] All interactive elements keyboard-reachable (Tab order logical)
- [ ] Visible focus indicator on every interactive element
- [ ] No keyboard trap (user can always leave any component)
- [ ] Touch targets ≥ 44×44px (mobile)

### Content
- [ ] Page has a descriptive `<title>`
- [ ] Headings used hierarchically (h1 → h6, no skipping)
- [ ] All images have meaningful `alt` text (or `alt=""` for decorative)
- [ ] Form inputs have associated `<label>`
- [ ] Error messages are descriptive and tied to the field

### ARIA
- [ ] ARIA used only when native HTML semantics are insufficient
- [ ] `aria-live` regions for dynamic content updates
- [ ] Modals trap focus and restore on close
- [ ] `role`, `aria-label`, `aria-describedby` used correctly

## Component Spec Template

```
## Component: <Name>

### Purpose
<what user problem this solves>

### Variants
- Default
- [other variants]

### States
| State | Visual | Interaction |
|---|---|---|
| Default | | |
| Hover | | |
| Focus | | |
| Active | | |
| Disabled | | |
| Loading | | |
| Error | | |
| Success | | |
| Empty | | |

### Props / API
| Prop | Type | Required | Default | Description |
|---|---|---|---|---|

### Accessibility
- Role: <HTML element or ARIA role>
- Keyboard: <Tab, Enter, Escape, Arrow keys behaviour>
- Screen reader: <what is announced>
- Focus management: <enter/exit behaviour>

### Tokens Used
- Colour: <list>
- Typography: <list>
- Spacing: <list>
- Shadow / radius: <list>

### Responsive Behaviour
- Mobile (< 768px): <description>
- Tablet (768–1024px): <description>
- Desktop (> 1024px): <description>

### Motion / Animation
- Enter: <description>
- Exit: <description>
- Interaction feedback: <description>
- Reduced motion: <fallback>
```

## CLI/TUI UX Heuristics

CLI tools are user interfaces too. Apply these principles:

| Heuristic | Implementation |
|---|---|
| **Discoverability** | `--help` on every command; top-level `--help` lists all commands with one-line descriptions |
| **Progressive disclosure** | Short help for common path; `--help --verbose` for full reference |
| **Structured output** | `--output json` for machine consumption; table for humans |
| **Error messages** | Say what went wrong, why, and how to fix it. Never "error: unknown error" |
| **Exit codes** | Document them; 0 = success always; non-zero values are stable |
| **Confirmation** | Destructive actions require explicit confirmation or `--force` flag |
| **Colour** | Use colour meaningfully; always degrade gracefully with `NO_COLOR` |

## Colour System

### Design Token Naming Convention
```
color-{semantic}-{variant}-{state}

Examples:
color-primary-default        // primary action, resting
color-primary-hover          // primary action, hovered
color-surface-elevated       // raised surface (card, modal)
color-text-subtle            // secondary text
color-feedback-error-subtle  // error background tint
```

### Accessible Palette Construction
1. Define base scale (50–950 for each hue)
2. Map semantic tokens to scale values
3. Verify all text/background pairs meet contrast requirements
4. Define dark mode mappings separately
5. Never hard-code hex in components — always use tokens

## User Journey Map Template

```
## Journey: <Name>
## Persona: <Name>
## Goal: <what they want to achieve>

| Stage | User Action | System Response | Emotion | Pain Points | Opportunities |
|---|---|---|---|---|---|
| Awareness | | | | | |
| Discovery | | | | | |
| Onboarding | | | | | |
| First use | | | | | |
| Regular use | | | | | |
| Recovery | | | | | |
```

## References

- WCAG 2.1: https://www.w3.org/TR/WCAG21/
- WCAG 2.2: https://www.w3.org/TR/WCAG22/
- ARIA Authoring Practices Guide: https://www.w3.org/WAI/ARIA/apg/
- Material Design 3: https://m3.material.io/
- Apple HIG: https://developer.apple.com/design/human-interface-guidelines/
- Nielsen Norman Group: https://www.nngroup.com/
- Refactoring UI (Wathan & Schoger)
- The Design of Everyday Things (Norman)
