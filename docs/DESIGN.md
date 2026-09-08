# Burrow 0.2 — native visual specification

Reference inspected: https://mole.fit/ and its published Clean/Status screenshots.
The relevant direction is a dark, restrained canvas with compact top navigation,
strong hierarchy, generous whitespace and a simple focal point. Not a clone of
the website, its solar-system artwork, or its additional product features.

Two generated concept boards were rejected: both introduced unrelated features
and a light sidebar. They are not production specifications and are not shipped.
The explicit specification below and the live reference govern implementation.

## Shared tokens

Background #111519; surfaces #191F24; border #313C42; hover #242D33;
text #EDF2F0; muted #A1B2B5; mint #97E4C0; text on mint #102C21;
warning #F4C57A; danger #FF9A9A. Native window controls remain intact.

System sans-serif fonts are loaded locally, validated, and never distributed.
Use Segoe UI on Windows, SF/Arial on Mac, and DejaVu Sans on Linux QA; fall back
to egui's included fonts if unavailable. Headings 28–36 logical pixels, controls
13–14, details 11–12. Borders 1px, panel radius 16, button radius 11, content
width at most 900px. Keyboard focus remains visible. Text scale uses egui zoom.

## Screens and states

- Overview: centered static orbital emblem; “Room to breathe.”; two actions;
  CPU and RAM readings; a drive list with free/total/used labels outside bars.
  Unknown and stale readings are explicit. No health score or guessed values.
- Clean up: centered scan entry; age choice; allowlist explanation. Results use
  a cached byte total, filter, visible selection, clear selection, separate
  review, virtualized filename/path/size rows, and a confirmation acknowledgement.
  Empty, filtered-empty, partial, busy, cancelled and failure states are explicit.
- Disk explorer: centered read-only folder picker; result summary and virtualized
  rows. Copy path is the only per-file action. No delete or permanent removal.
- About & help: the same typography and surfaces, operating boundaries, keyboard
  controls, privacy details, installation and update links. No extra Tools page.

Top navigation remains visible. Compact mode uses shorter tab labels at narrow
logical widths. Content and confirmation windows scroll rather than hiding
controls at increased text scale. There is no whole-app backdrop blur, looping
animation, remote asset, browser runtime or raster screenshot used as UI.

## QA comparison points

Inspect screenshots at 1060x800 and 720x560 plus enlarged text. Check layout,
palette, typography, spacing, navigation, row truncation, capacity labels and
empty/confirmation states against this specification. Pointer and keyboard tests
are distinct from screenshot inspection. Record deviations honestly; never call
an unrelated generated concept an approved or matched design.
