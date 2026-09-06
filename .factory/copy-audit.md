# Landing-page copy audit

Audited 6 September 2026. Counts use space-separated words. No sentence is
over 22 words and no banned plain-words term appears.

| Copy | Words | Result |
| --- | ---: | --- |
| For small web teams and accessibility consultants, find blocked keyboard controls before users report them. | 14 | Pass |
| See a populated local report without setup. | 8 | Pass |
| Free CLI. | 2 | Pass |
| No account is required. | 5 | Pass |
| Reports and screenshots stay on your machine or CI runner. | 10 | Pass |
| Uses Chromium desktop zoom at 200% and 400%. | 8 | Pass |
| Use a real keyboard path. | 6 | Pass |
| At 200% and 400%. | 4 | Pass |
| Open the local report. | 5 | Pass |
| Record a path once, then keep one report for each selected zoom level. | 13 | Pass |
| The sample writes its own temporary folder and reports known blocked controls. | 11 | Pass |
| It exits 1 because those findings are intentional. | 8 | Pass |
| The sample checks a checkout flyout with a payment button hidden below its fixed edge. | 15 | Pass |
| Static audits do not replay a task. | 7 | Pass |
| This runner collects geometry and focus evidence after each key. | 10 | Pass |
| Checks whether focus reached the expected control and whether that control has a name. | 15 | Pass |
| Records whether the focused control has an outline or box-shadow treatment. | 12 | Pass |
| Checks the viewport, overflow ancestors, and sampled points over the focused control. | 12 | Pass |
| Records whether the document or a containing region can scroll to expose focus. | 14 | Pass |
| It does not certify WCAG conformance or replace manual testing. | 10 | Pass |
| It does not upload reports. | 5 | Pass |
| Review local screenshots before sharing them because they can contain sensitive interface content. | 13 | Pass |
| Local CLI checks keyboard workflows at browser zoom. | 8 | Pass |

## Terminology

| Concept | Term used |
| --- | --- |
| User’s keyboard path | workflow |
| A stop reached by a key | focus stop |
| Saved HTML and JSON evidence | report |
| Shipped checkout example | bundled sample |
| Chromium browser magnification | desktop zoom |
| A serious result | blocking finding |
