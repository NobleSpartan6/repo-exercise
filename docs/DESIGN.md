# Interface design

## Structure

Five centered tabs—Clean, Apps, Optimize, Analyze, Status—match the reference's
workflow. A separate help button keeps the main navigation short. Workspaces use
the same controls, spacing, typography, and confirmation patterns on Mac and
Windows; native title-bar and window controls remain with the OS.

Clean emphasizes review. Apps uses searchable rows and a selected-app detail area.
Optimize presents named tasks, not a vague speed-up score. Analyze gives space to
a proportional map and breadcrumbs. Status groups live readings above a process
list. Compact and enlarged-text layouts scroll rather than hiding primary actions.

## Visual language

| Role | Color |
| --- | --- |
| Background | `#111519` |
| Panel | `#191F24` |
| Raised control | `#242D33` |
| Border | `#313C42` |
| Text | `#EDF2F0` |
| Secondary text | `#A1B2B5` |
| Mint accent | `#97E4C0` |
| Warning | `#F4C57A` |
| Error | `#FF9A9A` |

UI text and controls are real native widgets, not a screenshot. Buttons have
visible focus states. System typography has a bundled-library fallback when the
OS font cannot be read. Lists truncate long paths with a full-path tooltip.

## Differences from Mole

Burrow keeps its own name, icon, charcoal/mint colors, and static accents. It does
not copy the reference's planetary artwork or pretend unsupported hardware data
exists. The miniature monitor is a separate floating window. Missing features and
system handoffs are listed in [Feature coverage](FEATURES.md).

## Review each change

Inspect desktop, compact, and 150% text captures. Check navigation, primary actions,
long names, dense lists, empty states, unavailable data, progress, and confirmation.
Use UI tests for click behavior and the native smoke tools for GPU rendering. A
successful compiler run is not a visual review.
