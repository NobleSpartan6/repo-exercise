from pathlib import Path
import base64, json, subprocess, tomllib

def replace(path, old, new, count=1):
    p=Path(path);s=p.read_text();assert s.count(old)==count, (path,old,s.count(old));p.write_text(s.replace(old,new))

replace('Cargo.toml','"dep:ab_glyph",','"dep:ab_glyph", "dep:image",')
replace('Cargo.toml',', "__screenshot"','')
replace('Cargo.toml','[dependencies]\n','[dependencies]\nimage = { version = "=0.25.10", optional = true, default-features = false, features = ["png"] }\n')
p=Path('Cargo.lock');s=p.read_text();before=tomllib.loads(s)
a=s.index('name = "gpu-allocator"');b=s.index('[[package]]',a);part=s[a:b];assert part.count('"windows 0.56.0"')==1
s=s[:a]+part.replace('"windows 0.56.0"','"windows 0.58.0"')+s[b:]
a=s.index('name = "burrow"');b=s.index('[[package]]',a);part=s[a:b];assert part.count(' "filetime",')==1
s=s[:a]+part.replace(' "filetime",',' "filetime",\n "image",')+s[b:];p.write_text(s)
assert {(x['name'],x['version'],x.get('checksum')) for x in before['package']} == {(x['name'],x['version'],x.get('checksum')) for x in tomllib.loads(s)['package']}
replace('src/main.rs','mod design;','mod design;\nmod qa;')
replace('src/ui.rs','smoke: Option<(usize, Instant)>','smoke: Option<crate::qa::NativeCheck>')
replace('src/ui.rs','Self::with_context(&cc.egui_ctx, !smoke, smoke)','Self::with_context(&cc.egui_ctx, true, smoke)')
replace('src/ui.rs','smoke: smoke.then(|| (0, Instant::now())),','smoke: smoke.then(crate::qa::NativeCheck::new),')
replace('src/ui.rs','    fn draw(&mut self, ctx: &egui::Context) {\n','''    fn draw(&mut self, ctx: &egui::Context) {
        if let Some(mut smoke) = self.smoke.take() {
            if let Some(page) = smoke.step(ctx) {
                self.navigate([Page::Overview, Page::Cleanup, Page::Explorer, Page::About][page], ctx);
            }
            self.smoke = Some(smoke);
        }
''')
p=Path('src/ui.rs');s=p.read_text();a=s.index('        // Explicit read-only release smoke mode.');b=s.index('\n    }\n}',a);p.write_text(s[:a]+s[b:])
replace('src/ui.rs','use std::time::{Duration, Instant};','use std::time::Duration;')
replace('src/ui.rs','(Page::Overview, "Overview")','(Page::Overview, "Home")')
replace('src/ui.rs','ui.spacing_mut().item_spacing.x = 3.0;', 'ui.spacing_mut().item_spacing.x = 3.0;\n                                ui.spacing_mut().button_padding = egui::vec2(10.0, 8.0);')
replace('src/ui.rs','design::record(&response, page.title());','let response = response.on_hover_text(page.title());\n                                        design::record(&response, page.title());')
marker='.on_hover_text("Select this file");'
replace('src/ui.rs',marker,marker+'''\n                        response.widget_info(|| egui::WidgetInfo::selected(
                            egui::WidgetType::Checkbox, ui.is_enabled(), checked,
                            &format!("Select {}", file.path.display())));''')
replace('src/ui_tests.rs','let r = rect(&ctx, name);','let r = rect(&ctx, name);\n                assert!(r.height() <= 40.0, "Navigation wraps at {size:?}: {name} {r:?}");')
replace('src/latest.rs','self.value.try_lock().ok()?.take()', '''match self.value.try_lock() {
            Ok(mut value) => value.take(),
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner().take(),
            Err(std::sync::TryLockError::WouldBlock) => None,
        }''')
p=Path('src/latest.rs');s=p.read_text();a=s.rfind('\n}');p.write_text(s[:a]+'''
    #[test]
    fn poisoned_writer_does_not_permanently_silence_readings() {
        let mailbox = Latest::default();
        mailbox.publish(7);
        let writer = mailbox.clone();
        assert!(std::thread::spawn(move || {
            let _guard = writer.value.lock().unwrap();
            panic!("simulated worker failure while locked");
        }).join().is_err());
        assert_eq!(mailbox.take(), Some(7));
        mailbox.publish(8);
        assert_eq!(mailbox.take(), Some(8));
    }
'''+s[a:])
for name in ['README.md','docs/RELEASE_NOTES.md','docs/PERFORMANCE.md','docs/INSTALL.md']:
    replace(name,'The pinned eframe diagnostic environment variable `EFRAME_SCREENSHOT_TO` captures the native window and exits when explicitly set by a maintainer; normal launches do not capture or save screenshots.', 'Native QA uses an explicit `--smoke-test` launch with `BURROW_SMOKE_OUTPUT` pointing to an empty evidence directory. It captures the app\'s GPU surface using egui screenshot events, not the unsupported eframe screenshot environment variable. Normal launches never capture or save screenshots.')
subprocess.run(['cargo','fmt','--all'],check=True)
subprocess.run(['cargo','metadata','--locked','--format-version','1'],check=True,stdout=subprocess.DEVNULL)
subprocess.run(['python3','-m','unittest','discover','-s','scripts','-p','test_*.py'],check=True)
entries=[]
for name in subprocess.check_output(['git','diff','--name-only','-z']).decode().split('\0'):
    if not name:continue
    data=Path(name).read_bytes()
    obj=json.loads(subprocess.check_output(['gh','api','--method','POST','repos/NobleSpartan6/burrow/git/blobs','--input','-'],input=json.dumps({'content':base64.b64encode(data).decode(),'encoding':'base64'}).encode()))
    entries.append({'path':name,'mode':'100644','type':'blob','sha':obj['sha']})
print('REPAIRED_ENTRIES='+json.dumps(entries),flush=True)
