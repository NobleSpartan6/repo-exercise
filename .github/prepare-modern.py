"""One-shot authoring helper. Never updates a branch or publishes a release."""
from pathlib import Path
import base64, json, os, subprocess
repo = 'NobleSpartan6/burrow'
assert os.environ['GITHUB_REPOSITORY'] == repo
assert os.environ['GITHUB_REF'] == 'refs/heads/release-prep/0.2.0-modern'
parent = 'f30ef4fd49af77d3959f8e8357d20efc960b8085'
p = Path('Cargo.lock'); s = p.read_text()
assert s.count('name = "burrow"\nversion = "0.1.1"') == 1
s = s.replace('name = "burrow"\nversion = "0.1.1"', 'name = "burrow"\nversion = "0.2.0"', 1)
pos = s.index('dependencies = [', s.index('name = "burrow"')) + len('dependencies = [')
s = s[:pos] + '\n "ab_glyph",' + s[pos:]; p.write_text(s)
p = Path('src/engine.rs'); p.write_text(p.read_text() + '\n#[cfg(all(test, any(windows, target_os = "macos")))]\n#[path = "native_tests.rs"]\nmod native_tests;\n')
for p in [Path('packaging/windows.iss'),Path('SECURITY.md'),*Path('docs').glob('*.md')]:
    p.write_text(p.read_text().replace('NobleSpartan6/repo-exercise','NobleSpartan6/burrow'))
p=Path('docs/INSTALL.md');p.write_text(p.read_text().replace('0.1.1','0.2.0').replace('the sidebar','the footer'))
p=Path('scripts/package.py');p.write_text(p.read_text().replace('(244, 252, 248, 255)','(151, 228, 192, 255)').replace('(19, 124, 102, 255)','(17, 21, 25, 255)'))
p=Path('CHANGELOG.md');p.write_text('# Changelog\n\n## 0.2.0\n\nModern native dark interface, top navigation, system typography, scrollable compact layouts, virtualized rows and cached totals. Expanded native startup, confirmation, packaging, installer and disposable Trash recovery tests. Updated repository links. No broader cleanup permissions, browser runtime, or decorative animation loop.\n\n'+p.read_text().replace('# Changelog\n','',1))
p=Path('docs/PERFORMANCE.md');p.write_text(p.read_text()+'\n## 0.2 interface budget\n\nNo continuously animated decorations, blur, web view or remote assets. Static orbital geometry. System fonts are validated and loaded once, not distributed. Preview byte totals are cached on receipt. Both file lists are virtualized inside finite-height scroll regions. Scan progress repaints occur at about 7 Hz; CPU/RAM sampling at 0.5 Hz; monitoring does not request repaints on other pages. CI idle measurements are observations on a single machine, not Mac/Windows hardware guarantees.\n')
p=Path('.github/workflows/burrow.yml');s=p.read_text().replace('scrot\n','scrot fonts-dejavu-core\n').replace('run: cargo clippy --locked --all-targets','run: cargo clippy --locked --all-targets -- -D warnings')
s=s.replace('      - name: Check Rust lints','      - name: Verify one disposable native Trash round trip\n        if: runner.os != \'Linux\'\n        run: cargo test --locked --lib engine::native_tests::native_trash_roundtrip -- --ignored --exact\n      - name: Check Rust lints')
s=s.replace('      - uses: actions/upload-artifact@v4\n        if: runner.os !=', '''      - name: Fetch verified old Windows installer
        if: runner.os == 'Windows'
        env:
          GH_TOKEN: ${{ github.token }}
        run: gh release download v0.1.1-preview-f30ef4f --pattern '*Setup.exe' --dir "$RUNNER_TEMP/old-burrow"
      - name: Test Windows installation upgrade launch and uninstall
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          $env:BURROW_OLD_INSTALLER = (Get-ChildItem "$env:RUNNER_TEMP/old-burrow/*Setup.exe").FullName
          python scripts/verify_install.py
          if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
      - name: Test copied Mac application from disk image
        if: runner.os == 'macOS'
        run: python3 scripts/verify_install.py '${{ matrix.package }}'
      - uses: actions/upload-artifact@v4
        if: runner.os !=''')
s=s.replace("if: always() && runner.os == 'Linux'",'if: always()').replace('name: native-gui-QA','name: native-QA-${{ matrix.label }}');p.write_text(s)
subprocess.run(['cargo','fmt','--all'],check=True)
subprocess.run(['python3','-m','unittest','discover','-s','scripts','-p','test_*.py'],check=True)
def api(endpoint,body):
    return json.loads(subprocess.check_output(['gh','api','--method','POST',f'repos/{repo}/{endpoint}','--input','-'],input=json.dumps(body).encode()))
entries=[]
for name in subprocess.check_output(['git','diff','--name-only','-z']).decode().split('\0'):
    if not name: continue
    data=Path(name).read_bytes()
    sha=api('git/blobs',{'content':base64.b64encode(data).decode(),'encoding':'base64'})['sha']
    entries.append({'path':name,'mode':'100644','type':'blob','sha':sha})
for name in ['.github/workflows/prepare-modern.yml','.github/prepare-modern.py']:
    entries.append({'path':name,'mode':'100644','type':'blob','sha':None})
base=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],text=True).strip()
tree=api('git/trees',{'base_tree':base,'tree':entries})['sha']
commit=api('git/commits',{'tree':tree,'parents':[parent],'message':'feat: redesign Burrow 0.2 with modern native UI and installation regression checks'})['sha']
print('PREPARED_COMMIT='+commit,flush=True)
print('PREPARED_TREE='+tree,flush=True)
