"""Tests for the public website generator (site/build.py).

Builds the site from the repository into a temporary directory and checks that every page
exists, every internal link and asset resolves, every repository link names a file or directory
that exists in the tree, the evidence and publication records are consistent, and no phrase of
the obsolete design (the 3U store-and-forward relay of the 2026-10-15 site) appears.
"""

import hashlib
import json
import re
import subprocess
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit

import pytest

REPO = Path(__file__).resolve().parents[2]
BUILD = REPO / "site" / "build.py"
BASE = "/SMART-QSO/"
BLOB = "https://github.com/cheesejaguar/SMART-QSO/blob/main/"
TREE = "https://github.com/cheesejaguar/SMART-QSO/tree/main/"

PAGES = ["index.html", "mission/index.html", "spacecraft/index.html", "engineering/index.html",
         "participate/index.html", "404.html"]
EVIDENCE = ["margins.json", "rid_summary.json", "schedule_actions.json"]

# Phrases of the obsolete design. "3U" is allowed only as the dispenser slot ("a 1U in a 3U slot")
# and in the ISIS antenna product name ("AntS 1U–3U").
OBSOLETE = [r"store-and-forward relay", r"Ed25519", r"12-month"]
ALLOWED_3U = [r"3U slot", r"1U[–-]3U"]


class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links: list[str] = []
        self.ids: set[str] = set()
        self.imgs_without_alt: list[str] = []

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if "id" in a:
            self.ids.add(a["id"])
        for key in ("href", "src"):
            if a.get(key):
                self.links.append(a[key])
        if tag == "img" and not a.get("alt"):
            self.imgs_without_alt.append(a.get("src", "?"))


@pytest.fixture(scope="module")
def site(tmp_path_factory):
    out = tmp_path_factory.mktemp("site")
    proc = subprocess.run([sys.executable, str(BUILD), "--out", str(out), "--base", BASE],
                          capture_output=True, text=True, cwd=REPO)
    assert proc.returncode == 0, proc.stderr
    return out


def parse(path: Path) -> Links:
    p = Links()
    p.feed(path.read_text(encoding="utf-8"))
    return p


def visible_text(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    text = re.sub(r"<(script|style)\b.*?</\1>", " ", text, flags=re.S)
    return re.sub(r"<[^>]+>", " ", text)


def test_every_page_exists(site):
    for page in PAGES:
        assert (site / page).is_file(), page
    for extra in ["sitemap.xml", "robots.txt", "favicon.svg", ".nojekyll", "assets/site.css",
                  "publication.json", "checksums.sha256"]:
        assert (site / extra).is_file(), extra
    for name in EVIDENCE:
        assert (site / "evidence" / name).is_file(), name


def test_internal_links_resolve(site):
    problems = []
    for html_file in site.rglob("*.html"):
        rel = html_file.relative_to(site).as_posix()
        for link in parse(html_file).links:
            parts = urlsplit(link)
            if parts.scheme or parts.netloc or link.startswith(("mailto:", "#")):
                continue
            path = unquote(parts.path)
            if path.startswith("/"):
                assert path.startswith(BASE), f"{rel}: absolute link {link} outside the base path"
                target = site / path[len(BASE):]
            else:
                target = (html_file.parent / path)
            if path.endswith("/") or target.is_dir():
                target = target / "index.html"
            if not target.resolve().is_file():
                problems.append(f"{rel} -> {link}")
            elif parts.fragment and target.suffix == ".html":
                if parts.fragment not in parse(target.resolve()).ids:
                    problems.append(f"{rel} -> {link} (no such anchor)")
    assert not problems, "\n".join(problems)


def test_repository_links_exist(site):
    problems, count = [], 0
    for html_file in site.rglob("*.html"):
        for link in parse(html_file).links:
            for prefix in (BLOB, TREE):
                if link.startswith(prefix):
                    count += 1
                    path = unquote(urlsplit(link).path.split("/main/", 1)[1])
                    if not (REPO / path).exists():
                        problems.append(f"{html_file.relative_to(site)} -> {path}")
    assert count > 50
    assert not problems, "\n".join(problems)


def test_no_external_resources(site):
    """Pages load nothing from another host: no CDN, no web fonts, no remote images."""
    for html_file in site.rglob("*.html"):
        text = html_file.read_text(encoding="utf-8")
        for m in re.finditer(r'<(?:link|script|img|source|iframe)\b[^>]*(?:href|src)="([^"]+)"', text):
            if 'rel="canonical"' in m.group(0):
                continue    # a URL, not a request
            assert not re.match(r"^(?:https?:)?//", m.group(1)), f"{html_file.name}: {m.group(1)}"
    css = (site / "assets/site.css").read_text(encoding="utf-8")
    assert "@import" not in css and "url(" not in css


def test_no_obsolete_phrases(site):
    problems = []
    for html_file in site.rglob("*.html"):
        text = visible_text(html_file)
        for phrase in OBSOLETE:
            if re.search(phrase, text, re.I):
                problems.append(f"{html_file.relative_to(site)}: {phrase}")
        stripped = text
        for allowed in ALLOWED_3U:
            stripped = re.sub(allowed, " ", stripped)
        if re.search(r"\b3U\b", stripped):
            problems.append(f"{html_file.relative_to(site)}: 3U")
    assert not problems, "\n".join(problems)


def test_no_status_overclaim(site):
    """Nothing says the satellite is in orbit or that its frequencies are coordinated."""
    for page in PAGES:
        text = visible_text(site / page).lower()
        for claim in ("is in orbit", "now in orbit", "frequencies are coordinated", "has launched",
                      "was launched", "frequency coordinated"):
            assert claim not in text, f"{page}: {claim!r}"
    participate = visible_text(site / "participate/index.html")
    assert "not in orbit" in participate
    assert "Not submitted" in participate


def test_accessibility_basics(site):
    for page in PAGES:
        html = (site / page).read_text(encoding="utf-8")
        assert '<html lang="en">' in html
        assert 'name="viewport"' in html
        assert html.count("<h1") == 1, page
        assert '<main id="main"' in html and 'class="skip"' in html
        assert not parse(site / page).imgs_without_alt, page
    css = (site / "assets/site.css").read_text(encoding="utf-8")
    assert "prefers-color-scheme: dark" in css and ":focus-visible" in css


def test_evidence_matches_the_documents(site):
    rid = json.loads((site / "evidence/rid_summary.json").read_text())
    assert sum(d["final"] for d in rid["dispositions"]) == rid["total"]
    report = (REPO / "docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md").read_text(encoding="utf-8")
    assert f"{rid['total']} RIDs" in report
    margins = json.loads((site / "evidence/margins.json").read_text())
    assert len(margins["rows"]) >= 10 and margins["rows"][0]["quantity"] == "Mass"
    actions = json.loads((site / "evidence/schedule_actions.json").read_text())
    assert actions["actions"][0]["id"] == "DA-01"
    assert all(a["due_dates"] for a in actions["actions"])


def test_publication_record(site):
    pub = json.loads((site / "publication.json").read_text())
    assert pub["base_path"] == BASE
    for rel, digest in pub["artifacts"].items():
        assert hashlib.sha256((site / rel).read_bytes()).hexdigest() == digest, rel
    for line in (site / "checksums.sha256").read_text().splitlines():
        digest, rel = line.split("  ", 1)
        assert hashlib.sha256((site / rel).read_bytes()).hexdigest() == digest, rel
    for path in pub["sources"]:
        assert (REPO / path).is_file(), path
    # the obsolete artifacts of the 2026-10-15 site are gone
    for old in ("proposal.pdf", "SMART-QSO-reference.zip", "spacecraft-concept.png"):
        assert not (site / old).exists()


def test_sitemap_lists_the_pages(site):
    sitemap = (site / "sitemap.xml").read_text()
    for page in PAGES[:-1]:
        loc = "https://cheesejaguar.github.io/SMART-QSO/" + page.replace("index.html", "")
        assert f"<loc>{loc}</loc>" in sitemap


def test_missing_table_fails_loudly(tmp_path, monkeypatch):
    """A document that loses a table the site needs stops the build; nothing is published."""
    sys.path.insert(0, str(REPO / "site"))
    import build  # noqa: E402

    real = build.read

    def broken(path):
        text = real(path)
        if path == build.CDR:
            text = text.replace("| Quantity | Requirement |", "| Quantity | Needed |")
        return text

    monkeypatch.setattr(build, "read", broken)
    build._cache.clear()
    out = tmp_path / "out"
    assert build.main(["--out", str(out)]) == 1
    assert not out.exists()
    build._cache.clear()
