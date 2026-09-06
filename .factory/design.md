# Visual thesis: the focus inspector's field sheet

## Direction and rationale

Zoom Workflow Check uses a **risograph tactile collage**: overprinted ink,
registration slips, crop marks, torn-paper edges, and a loupe moving across a
keyboard path. Accessibility reports often look sterile and falsely final. A
field-sheet aesthetic instead makes the product feel like what it is: repeatable
evidence assembled by a practitioner, with every focus stop visibly annotated.
Decoration always explains either magnification, focus order, or local evidence.

This is intentionally a single light, ink-on-paper mode. A dark theme would
break the physical-paper premise; explicit deep-ink surfaces provide contrast
where a reversed treatment is useful.

## Tokens

- Paper `#F3EBD8`: page background, warm rather than clinical.
- Clean paper `#FFF9EC`: primary reading surface.
- Registration ink `#18221D`: body text and structural strokes.
- Muted ink `#5C625A`: secondary copy (7.0:1 on paper).
- Riso blue `#095D9A`: links and controls (5.7:1 on paper).
- Signal orange `#D94B21`: attention marks, never body copy.
- Leaf `#266444`: passing evidence.
- Bruise `#812D42`: blocking findings and errors.
- Safety yellow `#F2C94C`: focus halo paired with a dark outer keyline.

Surfaces use solid ink and coarse halftones, never gradients. Registration
offsets are limited to 2–4 px so text remains crisp.

## Type and spacing

Two local system stacks avoid font downloads: `Arial Black` / `Arial Narrow`
for condensed poster headings, and `ui-monospace` for evidence, code, and body
labels. The six-step scale is 0.78, 0.9, 1, 1.25, 2, and clamp(2.8–5rem).
Body copy is at least 16 px with 1.55 leading and a 68-character measure.

Spacing follows a 4/8 px rhythm: 4, 8, 12, 16, 24, 32, 48, 72, and 96 px.
The desktop is an asymmetric two-column paste-up. At 760 px it becomes one
column, moves proof before secondary context, and drops non-essential crop-mark
ornament. All controls keep a 44 px minimum target.

## Interaction grammar

Interactive elements behave like stamped paper tabs: a 2 px ink keyline, a
3 px offset shadow, then a 2 px press translation. The focus indicator is a
3 px safety-yellow ring plus 2 px registration-ink separation. Status is always
expressed with icon, word, and color. Code copy provides immediate live-region
feedback. The report comparison reads left-to-right at desktop and in zoom-level
sequence on mobile.

## Motion

Only evidence enters: paper strips settle upward by 8 px over 220 ms and the
hero loupe shifts once by 6 px over 280 ms. No looping motion or parallax.
`prefers-reduced-motion: reduce` removes transforms and sets transitions and
animations to effectively instant. State changes retain borders, labels, and
layering, so depth does not depend on motion.

## Original asset plan and provenance

- `site/public/zoom-field-sheet.webp`: generated specifically for this project
  with `/opt/fleet/lib/gen-image.sh`, deployment `factory-image`, then resized
  and converted locally to WebP. Prompt: “Editorial risograph print on warm
  recycled paper, close-up magnifying loupe enlarging a web dialog and keyboard
  focus path, cobalt blue and vermilion overprint, imperfect registration,
  coarse halftone grain, torn-paper collage edges, crop marks, wide composition,
  no words, no logos, no gradients, no photorealism.” Original work under the
  project MIT license; generator metadata is stored beside the source artifact.
- `site/public/focus-mark.svg` and the tiny focus-path/status marks are
  hand-authored geometric SVG/CSS, not stock icons.
- `site/public/zoom-field-sheet-og.webp` and `apple-touch-icon.png` are local
  crops of the generated field-sheet artwork for sharing and device icons; no
  new external image source was added.

The generated image clarifies the core idea—magnification exposing a blocked
keyboard path—and is the only raster illustration. Report thumbnails are user
evidence and remain local.
