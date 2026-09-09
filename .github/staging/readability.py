from pathlib import Path
p=Path('src/design.rs');s=p.read_text();old='    style.spacing.interact_size.y = 36.0;';assert s.count(old)==1;s=s.replace(old,old+'\n    style.spacing.scroll = egui::style::ScrollStyle::solid();\n    style.spacing.scroll.bar_width = 8.0;');p.write_text(s)
p=Path('src/workspaces.rs');s=p.read_text()
s=s.replace('Live readings. No made-up health score or automatic fixes.','CPU, memory, network and process activity.')
a='''            for disk in &self.drive_sample.disks{
                design::card().inner_margin(12).show(ui,|ui|{'''
b='''            let drive_columns=if ui.available_width()>540.0{2}else{1};
            egui::ScrollArea::vertical().id_salt("status-volumes").max_height(145.0).show(ui,|ui|{
                for row in self.drive_sample.disks.chunks(drive_columns){
                    ui.columns(drive_columns,|columns|{for(i,disk)in row.iter().enumerate(){
                        let ui=&mut columns[i];
                        design::card().inner_margin(12).show(ui,|ui|{
                            ui.set_min_width(ui.available_width());'''
assert s.count(a)==1;s=s.replace(a,b)
a='''                });
            }
            ui.add_space(16.0);design::title(ui,"Processes",19.0);'''
b='''                        });
                    }});
                    ui.add_space(5.0);
                }
            });
            ui.add_space(12.0);design::title(ui,"Processes",19.0);'''
assert s.count(a)==1;s=s.replace(a,b)
a='''                design::card().inner_margin(14).show(ui,|ui|{
                    let mut selected=self.workspace.maintenance_selected.contains(&action);'''
b='''                design::card().inner_margin(14).show(ui,|ui|{
                    ui.set_min_width(ui.available_width());
                    let mut selected=self.workspace.maintenance_selected.contains(&action);'''
assert s.count(a)==1;s=s.replace(a,b);p.write_text(s)
