import base64, hashlib, json, lzma
from pathlib import Path
raw=base64.b64decode(''.join(Path(f'.github/staging/{i}.b64').read_text() for i in range(8)),validate=True)
assert hashlib.sha256(raw).hexdigest()=='9c1dfbe62a292a68264cbea9dafdb13605e90a0f80929df9e4187194865e461e'
payload=json.loads(lzma.decompress(raw))
for name, patch in payload.items():
    p=Path(name)
    assert not p.is_absolute() and '..' not in p.parts and '.git' not in p.parts
    old=p.read_bytes() if p.exists() else b''
    assert hashlib.sha256(old).hexdigest()==patch['old'], name
    lines=old.decode().splitlines(keepends=True)
    for a,b,replacement in reversed(patch['changes']): lines[a:b]=[replacement]
    new=''.join(lines).encode()
    assert hashlib.sha256(new).hexdigest()==patch['new'], name
    p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(new)
# Post-review corrections: inexpensive immutable review snapshots, no stale app
# selection, and closing an embedded monitor must not close the main window.
p=Path('src/software.rs');s=p.read_text()
s=s.replace('use std::path::{Path, PathBuf};','use std::path::{Path, PathBuf};\nuse std::sync::Arc;')
s=s.replace('manifest:Vec<BundleStamp>','manifest:Arc<Vec<BundleStamp>>')
s=s.replace('let manifest=bundle_manifest(&path,control)?;', 'let manifest=Arc::new(bundle_manifest(&path,control)?);')
s=s.replace('bundle_manifest(&path,control)?!=plan.manifest','bundle_manifest(&path,control)?.as_slice()!=plan.manifest.as_slice()')
s=s.replace('let manifest=bundle_manifest(&path,&Control::default()).unwrap();','let manifest=Arc::new(bundle_manifest(&path,&Control::default()).unwrap());')
p.write_text(s)
p=Path('src/workspaces.rs');s=p.read_text()
a='let response=ui.add_sized([w,47.0],egui::Button::new(egui::RichText::new(label).size(13.0)).selected(self.workspace.app_selected==Some(i)).wrap_mode(egui::TextWrapMode::Truncate));'
b='let response=ui.add_enabled_ui(self.busy.is_none(),|ui|ui.add_sized([w,47.0],egui::Button::new(egui::RichText::new(label).size(13.0)).selected(self.workspace.app_selected==Some(i)).wrap_mode(egui::TextWrapMode::Truncate))).inner;'
assert s.count(a)==1;s=s.replace(a,b)
a='open.store(false,Ordering::Relaxed);ctx.send_viewport_cmd(egui::ViewportCommand::Close);'
b='open.store(false,Ordering::Relaxed);if class!=egui::ViewportClass::Embedded{ctx.send_viewport_cmd(egui::ViewportCommand::Close);}'
assert s.count(a)==1;s=s.replace(a,b)
a='s.try_lock().ok().map(|s|s.clone())'
b='match s.try_lock(){Ok(value)=>Some(value.clone()),Err(std::sync::TryLockError::Poisoned(error))=>Some(error.into_inner().clone()),Err(std::sync::TryLockError::WouldBlock)=>None}'
assert s.count(a)==1;s=s.replace(a,b);p.write_text(s)
p=Path('docs/RELEASING.md');s=p.read_text().replace('0.2.0','0.3.0').replace('all four pages','all six screens').replace('Install developer prerequisites from README','Install developer prerequisites from docs/DEVELOPMENT.md').replace('begin with read-only Disk explorer','begin with read-only Analyze');p.write_text(s)
Path('.github/staging/paths.json').write_text(json.dumps(sorted(set(payload)|{'Cargo.lock','docs/RELEASING.md'})))
print('Applied and hash-verified 30 source changes and reviewed corrections.')
