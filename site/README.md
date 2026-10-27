# SMART-QSO Website

The public site at <https://cheesejaguar.github.io/SMART-QSO/>. It is generated from the repository documents: there is
no hand-maintained copy of any design value.

## Build and preview

```bash
python3 site/build.py && python3 -m http.server -d site/_build
# then open http://localhost:8000/
```

`site/build.py` uses the Python 3 standard library only. It writes to `site/_build/` (git-ignored; `--out DIR` for
another directory). Pages link to each other with relative URLs, so the preview works from any directory. The base path
on the server (`--base`, default `/SMART-QSO/`, or `$SITE_BASE`) and the public URL (`--site-url`, or `$SITE_URL`) are
used only where an absolute URL is needed: `404.html`, the canonical links and `sitemap.xml`. On the local preview the
404 page therefore loads without its stylesheet.

Test: `python -m pytest -q scripts/tests/test_site_build.py` (also run with the other repository script tests in
Program Checks). It builds into a temporary directory and checks that every page exists, every internal link resolves,
every repository link names a file in the tree, nothing loads from another host, the evidence and checksums are
consistent, and no phrase of the obsolete 3U design appears.

## Where the content comes from

The build reads the documents each time it runs:

| Page | Pulled from the documents | Written in `build.py` |
|------|---------------------------|-----------------------|
| Home | Key numbers from the baseline (`docs/BASELINE_PARAMETERS.md` §1–§8); planning date and LTAN from the CDR report §3.1; RID total from §6 | Mission summary, status, the AI section, the launch path |
| Mission | Launch chain (CDR report §3.1), the minute-by-minute timeline (§3.2), mission phases (`docs/operations/MISSION_TIMELINE.md` §2), operating modes (`mission/CONOPS.md` §3), milestones (`mission/SCHEDULE.md` §4) | The plain-language timeline |
| Spacecraft | Mass, power and envelope values from the baseline; design completion table (CDR report §5); the two CAD sheets from `hardware/mechanical/` | Subsystem summaries; the illustration, drawn from the CAD dimensions |
| Engineering | §1.2, §7.1–§7.4 of the CDR report, the RID summary block of §6, the margins of §3.3, the chair decisions of §4, the dated actions of `mission/SCHEDULE.md` §5, the mission simulation headline (`docs/test/SIL_MISSION_SIMULATION.md` §7) | Framing sentences |
| Participate | Authorization status (`docs/regulatory/SPECTRUM_LICENSE.md` §4), IARU request status (`docs/regulatory/IARU_COORDINATION.md` §1), bands from the baseline §7, the beacon format (`docs/BEACON_SPEC.md` §3) | What help is needed |

Hand-written sentences cite their document with `cite(path, phrase, ...)`: the phrase must still appear in that
document, or the build stops. A table, section or pattern the build needs that is missing also stops it, with a message
naming the document. The site cannot drift from the repository; it fails instead.

The build also writes `evidence/*.json` (margins, RID summary, dated actions, milestones, chair decisions,
authorization status, key values), `publication.json` (source commit, each source document's SHA-256 and revision, and
each published file's SHA-256) and `checksums.sha256`.

## Publishing

`.github/workflows/site.yml` builds the site on every pull request that touches `site/`, `docs/`, `mission/` or
`hardware/mechanical/` (build only), and on a push to `main` or a manual run it replaces the contents of the `gh-pages`
branch with the build, keeping `.nojekyll`, using the workflow's own `GITHUB_TOKEN`. GitHub Pages serves `gh-pages`.

## Design

One hand-written stylesheet (`site/site.css`), system fonts, no scripts, no external requests. Light and dark themes
follow `prefers-color-scheme`. Wide tables scroll inside their own region on narrow screens; the page itself never
scrolls sideways.
