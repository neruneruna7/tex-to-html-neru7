name: Convert TeX archives and deploy Pages

on:
  push:
    branches:
      - main
    paths:
      - "sources/**/*.tar.gz"
      - "convert_tex.rs"
      - ".github/workflows/pages.yml"
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: github-pages
  cancel-in-progress: true

jobs:
  build:
    name: Build HTML
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install TeX Live and make4ht
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            texlive-latex-base \
            texlive-latex-recommended \
            texlive-latex-extra \
            texlive-fonts-recommended \
            texlive-fonts-extra \
            texlive-science \
            texlive-pictures \
            texlive-bibtex-extra \
            texlive-extra-utils \
            tex4ht \
            ghostscript \
            imagemagick
      - name: Check TeX tools
        run: |
          which make4ht
          make4ht --version
          which htlatex
          htlatex --version || true

      - name: Install rust-script
        run: |
          cargo install rust-script

      - name: Convert tar.gz sources to HTML
        run: |
          set -euo pipefail
          shopt -s nullglob

          rm -rf work dist
          mkdir -p work dist

          archives=(sources/*.tar.gz)

          if [ ${#archives[@]} -eq 0 ]; then
            echo "No archives found under sources/*.tar.gz" >&2
            exit 1
          fi

          for archive in "${archives[@]}"; do
            echo "::group::Converting $archive"
            rust-script convert_tex.rs "$archive"
            echo "::endgroup::"
          done

      - name: Generate top index
        run: |
          set -euo pipefail

          cat > dist/index.html <<'EOF'
          <!doctype html>
          <html lang="ja">
          <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title>Converted TeX Documents</title>
            <style>
              :root { color-scheme: light; }
              body {
                background: white;
                color: black;
                font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                max-width: 960px;
                margin: 3rem auto;
                padding: 0 1rem;
                line-height: 1.6;
              }
              code {
                background: #f3f3f3;
                padding: 0.1rem 0.3rem;
                border-radius: 4px;
              }
              a {
                color: #0645ad;
              }
            </style>
          </head>
          <body>
            <h1>Converted TeX Documents</h1>
            <p>Generated from <code>sources/*.tar.gz</code>.</p>
            <ul>
          EOF

          for dir in dist/*/; do
            name="$(basename "$dir")"
            echo "      <li><a href=\"./$name/\">$name</a></li>" >> dist/index.html
          done

          cat >> dist/index.html <<'EOF'
            </ul>
          </body>
          </html>
          EOF

      - name: Configure Pages
        uses: actions/configure-pages@v5

      - name: Upload Pages artifact
        uses: actions/upload-pages-artifact@v4
        with:
          path: dist

  deploy:
    name: Deploy Pages
    runs-on: ubuntu-latest
    needs: build

    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}

    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4