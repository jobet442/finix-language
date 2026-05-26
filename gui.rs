#![cfg(feature = "gui")]

use eframe::{egui, App, Frame};
use crate::parser::Parser;
use crate::bytecode_compiler::BytecodeCompiler;
use crate::vm::{Vm, InterpretResult};
use crate::chunk::*;
use ast::printer::AstPrinter;
use ast::TypeChecker;
use lexer::lexer::Lexer;
use lexer::token::TokenType;

pub fn run_app() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Turbo Finix v3.0 - Java & Python Style")
            .with_inner_size([1150.0, 750.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Turbo Finix IDE",
        options,
        Box::new(|_cc| Box::new(FinixGui::new())),
    )
}

pub struct FinixGui {
    source_code: String,
    ast_string: String,
    tokens_string: String,
    disassembly_lines: Vec<(usize, String)>,
    vm: Vm,
    vm_chunk: Option<Chunk>,
    error_message: Option<String>,
    typecheck_message: Option<String>,
    
    // UI Retro states
    active_tab: Tab,
    execution_result: Option<InterpretResult>,
    show_help: bool,
    gui_components: Vec<GuiComponent>,

    // File Management
    current_file_path: String,
    show_save_dialog: bool,
    show_open_dialog: bool,
    show_new_project_dialog: bool,
    dialog_input_path: String,
}

#[derive(PartialEq)]
enum Tab {
    VmState,
    AstView,
    TokensView,
    GuiBuilder,
}

#[derive(Clone, PartialEq)]
enum GuiComponent {
    Button(String),
    Label(String),
    TextInput(String),
    ClassDef(String),
    FunctionDef(String),
    ModuleDef(String),
}

impl Default for FinixGui {
    fn default() -> Self {
        Self::new()
    }
}

impl FinixGui {
    pub fn new() -> Self {
        let default_source = r#"// Hello World Demonstration
println("Hello World!");
"#.to_string();

        let mut gui = Self {
            source_code: default_source,
            ast_string: String::new(),
            tokens_string: String::new(),
            disassembly_lines: Vec::new(),
            vm: Vm::new(),
            vm_chunk: None,
            error_message: None,
            typecheck_message: None,
            active_tab: Tab::VmState,
            execution_result: None,
            show_help: false,
            gui_components: Vec::new(),
            current_file_path: "untitled.fnx".to_string(),
            show_save_dialog: false,
            show_open_dialog: false,
            show_new_project_dialog: false,
            dialog_input_path: String::new(),
        };
        gui.compile_code();
        gui
    }

    fn compile_code(&mut self) {
        self.error_message = None;
        self.typecheck_message = None;
        self.execution_result = None;
        self.ast_string.clear();
        self.tokens_string.clear();
        self.disassembly_lines.clear();
        self.vm_chunk = None;

        // 1. Lexing
        let mut lexer = Lexer::new(&self.source_code);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.token_type == TokenType::Eof {
                break;
            }
            tokens.push(format!(
                "Token: {:-15} | Lexeme: {:15} | Line: {}, Col: {}",
                tok.token_type.to_string(),
                format!("\"{}\"", tok.lexeme),
                tok.pos.line,
                tok.pos.col
            ));
        }
        self.tokens_string = tokens.join("\n");

        // 2. Parsing
        let mut parser = Parser::new(&self.source_code);
        match parser.parse_program() {
            Ok(program) => {
                let printer = AstPrinter::new();
                self.ast_string = printer.print_program(&program);

                // 3. Typechecking
                let checker = TypeChecker::new();
                match checker.check(&program) {
                    Ok(_) => {
                        self.typecheck_message = Some("Typecheck: OK".to_string());
                    }
                    Err(type_errs) => {
                        let err_strs: Vec<String> = type_errs.iter().map(|e| e.to_string()).collect();
                        self.typecheck_message = Some(format!("Typechecker Errors:\n{}", err_strs.join("\n")));
                    }
                }

                // 4. Compiling to VM Chunk
                let compiler = BytecodeCompiler::new();
                match compiler.compile(&program) {
                    Ok(chunk) => {
                        self.disassembly_lines = disassemble_chunk_to_lines(&chunk);
                        self.vm_chunk = Some(chunk.clone());
                        self.vm.chunk = chunk;
                        self.vm.ip = 0;
                        self.vm.stack.clear();
                        self.vm.output.clear();
                        self.vm.is_finished = false;
                    }
                    Err(comp_err) => {
                        self.error_message = Some(format!("Compile Error: {}", comp_err));
                    }
                }
            }
            Err(parse_err) => {
                self.error_message = Some(format!("Parse Error: {}", parse_err));
            }
        }
    }

    fn run_all(&mut self) {
        if self.vm_chunk.is_some() {
            self.execution_result = Some(self.vm.run());
        }
    }

    fn step_vm(&mut self) {
        if self.vm_chunk.is_some() && !self.vm.is_finished {
            match self.vm.step() {
                Ok(finished) => {
                    if finished {
                        self.execution_result = Some(InterpretResult::Ok);
                    }
                }
                Err(err) => {
                    self.execution_result = Some(InterpretResult::RuntimeError(err));
                    self.vm.is_finished = true;
                }
            }
        }
    }

    fn reset_vm(&mut self) {
        if let Some(chunk) = &self.vm_chunk {
            self.vm.chunk = chunk.clone();
            self.vm.ip = 0;
            self.vm.stack.clear();
            self.vm.output.clear();
            self.vm.is_finished = false;
            self.execution_result = None;
        }
    }
}

impl App for FinixGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Enforce ALL fonts to be Monospace for that classic MS-DOS text look
        let mut style = (*ctx.style()).clone();
        for font_id in style.text_styles.values_mut() {
            font_id.family = egui::FontFamily::Monospace;
        }
        ctx.set_style(style);

        // Customize colors to exactly match the classic Turbo C++ DOS interface
        let mut visuals = egui::Visuals::dark();
        // Classic DOS intense blue background
        let retro_blue = egui::Color32::from_rgb(0, 0, 168);
        // Light gray for borders, menus, scrollbar thumbs
        let retro_gray = egui::Color32::from_rgb(168, 168, 168);
        // Black for high contrast menu text
        let retro_black = egui::Color32::BLACK;
        // White for text
        let retro_white = egui::Color32::from_rgb(255, 255, 255);
        // Yellow for status hotkeys bar and selections
        let retro_yellow = egui::Color32::from_rgb(168, 168, 0);
        // Bright cyan for active borders/highlights
        let retro_cyan = egui::Color32::from_rgb(0, 168, 168);

        visuals.window_fill = retro_blue;
        visuals.panel_fill = retro_blue;
        visuals.hyperlink_color = retro_cyan;

        // Custom borders and inactive scroll tracks
        visuals.widgets.noninteractive.bg_fill = retro_blue;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, retro_gray);

        // Inactive widgets (buttons, dropdowns, tabs) -> Light Gray with Black Text
        visuals.widgets.inactive.bg_fill = retro_gray;
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, retro_black);

        // Hovered widgets -> Yellow background with Black Text
        visuals.widgets.hovered.bg_fill = retro_yellow;
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, retro_black);

        // Active clicked widgets -> Cyan background with White Text
        visuals.widgets.active.bg_fill = retro_cyan;
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, retro_white);

        // Text selections -> Yellow background
        visuals.selection.bg_fill = retro_yellow;
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, retro_black);
        
        // Text input background
        visuals.extreme_bg_color = retro_black;

        ctx.set_visuals(visuals);

        // --- KEYBOARD SHORTCUTS (DOS Retro style bindings) ---
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_help = !self.show_help;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
            if self.current_file_path == "untitled.fnx" {
                self.show_save_dialog = true;
                self.dialog_input_path = self.current_file_path.clone();
            } else {
                let _ = std::fs::write(&self.current_file_path, &self.source_code);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F3)) {
            self.source_code = r#"let a = 1;
let b = 2;
println(a + b);
"#.to_string();
            self.current_file_path = "untitled.fnx".to_string();
            self.compile_code();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F9)) {
            self.compile_code();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.run_all();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F8)) {
            self.step_vm();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F10)) {
            self.reset_vm();
        }

        // --- TOP MENU BAR ---
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::default().fill(retro_gray).inner_margin(4.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().visuals.widgets.inactive.bg_fill = retro_gray;
                    ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, retro_black);

                    ui.menu_button("File", |ui| {
                        if ui.button("New Project").clicked() {
                            self.show_new_project_dialog = true;
                            self.dialog_input_path = "MyProject".to_string();
                            ui.close_menu();
                        }
                        if ui.button("New File (F3)").clicked() {
                            self.source_code = "println(\"Hello World!\");\n".to_string();
                            self.current_file_path = "untitled.fnx".to_string();
                            self.compile_code();
                            ui.close_menu();
                        }
                        if ui.button("Open File").clicked() {
                            self.show_open_dialog = true;
                            self.dialog_input_path = self.current_file_path.clone();
                            ui.close_menu();
                        }
                        if ui.button("Save (F2)").clicked() {
                            if self.current_file_path == "untitled.fnx" {
                                self.show_save_dialog = true;
                                self.dialog_input_path = self.current_file_path.clone();
                            } else {
                                let _ = std::fs::write(&self.current_file_path, &self.source_code);
                            }
                            ui.close_menu();
                        }
                        if ui.button("Save As...").clicked() {
                            self.show_save_dialog = true;
                            self.dialog_input_path = self.current_file_path.clone();
                            ui.close_menu();
                        }
                        if ui.button("Help (F1)").clicked() {
                            self.show_help = true;
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        if ui.button("Clear Code").clicked() {
                            self.source_code.clear();
                            self.compile_code();
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Run", |ui| {
                        if ui.button("Run Program (F5)").clicked() {
                            self.run_all();
                            ui.close_menu();
                        }
                        if ui.button("Trace/Step VM (F8)").clicked() {
                            self.step_vm();
                            ui.close_menu();
                        }
                        if ui.button("Reset Execution (F10)").clicked() {
                            self.reset_vm();
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Compile", |ui| {
                        if ui.button("Compile Code (F9)").clicked() {
                            self.compile_code();
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Debug", |ui| {
                        if ui.button("Show VM State").clicked() {
                            self.active_tab = Tab::VmState;
                            ui.close_menu();
                        }
                        if ui.button("Show AST View").clicked() {
                            self.active_tab = Tab::AstView;
                            ui.close_menu();
                        }
                        if ui.button("Show Tokens View").clicked() {
                            self.active_tab = Tab::TokensView;
                            ui.close_menu();
                        }
                        if ui.button("Show GUI Builder").clicked() {
                            self.active_tab = Tab::GuiBuilder;
                            ui.close_menu();
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(retro_black, "Turbo Finix IDE v3.0");
                    });
                });
            });

        // --- BOTTOM STATUS HOTKEYS BAR ---
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::default().fill(retro_yellow).inner_margin(4.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(retro_black, "F1 Help");
                    ui.separator();
                    ui.colored_label(retro_black, "F2 Save");
                    ui.separator();
                    ui.colored_label(retro_black, "F3 Open/Reset");
                    ui.separator();
                    ui.colored_label(retro_black, "F5 Run");
                    ui.separator();
                    ui.colored_label(retro_black, "F8 Step");
                    ui.separator();
                    ui.colored_label(retro_black, "F9 Compile");
                    ui.separator();
                    ui.colored_label(retro_black, "F10 Reset VM");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let lines_count = self.source_code.lines().count();
                        ui.colored_label(retro_black, format!("Lines: {:<4}", lines_count));
                    });
                });
            });

        // --- LEFT COLUMN: EDITOR WINDOW & OUTPUT WINDOW ---
        egui::SidePanel::left("editor_panel")
            .resizable(true)
            .default_width(550.0)
            .frame(egui::Frame::default().fill(retro_blue).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Editor Window frame
                    let mut file_name = self.current_file_path.clone();
                    if file_name.len() > 16 {
                        file_name.truncate(13);
                        file_name.push_str("...");
                    }
                    let title = format!("╔═[■]══════════════════════ Edit: {:<16} ══════════════════════[↕]═╗", file_name);
                    ui.colored_label(retro_gray, title);
                    
                    egui::Frame::none()
                        .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                        .fill(retro_blue)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_source("editor_scroll")
                                .max_height(360.0)
                                .show(ui, |ui| {
                                    let text_edit = egui::TextEdit::multiline(&mut self.source_code)
                                        .font(egui::TextStyle::Monospace)
                                        .text_color(retro_white)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(18)
                                        .lock_focus(true);
                                    let response = ui.add(text_edit);
                                    if response.changed() {
                                        self.compile_code();
                                    }
                                });
                        });
                    
                    ui.colored_label(retro_gray, "╚══════════════════════════════════════════════════════════════════════╝");
                    ui.add_space(10.0);

                    // Output Console Window frame
                    ui.colored_label(retro_gray, "╔════════════════════════════ Output Console ═════════════════════════╗");
                    
                    egui::Frame::none()
                        .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                        .fill(retro_blue)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            let console_text = self.vm.output.join("");
                            let mut console_display = if console_text.is_empty() {
                                "(No execution output yet. Press F5 to Run.)"
                            } else {
                                &console_text
                            }.to_string();

                            egui::ScrollArea::vertical()
                                .id_source("console_scroll")
                                .max_height(140.0)
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut console_display)
                                            .font(egui::TextStyle::Monospace)
                                            .text_color(retro_cyan)
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(6)
                                            .interactive(false)
                                    );
                                });
                        });
                        
                    ui.colored_label(retro_gray, "╚══════════════════════════════════════════════════════════════════════╝");
                });
            });

        // --- CENTRAL PANEL: DEBUGGER / COMPILATION WINDOW ---
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(retro_blue).inner_margin(8.0))
            .show(ctx, |ui| {
                // Tab Selection Bar (Styled like active/inactive items)
                ui.horizontal(|ui| {
                    ui.style_mut().visuals.widgets.inactive.bg_fill = retro_blue;
                    ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, retro_gray);
                    ui.style_mut().visuals.widgets.active.bg_fill = retro_cyan;

                    ui.selectable_value(&mut self.active_tab, Tab::VmState, " VM Watch ");
                    ui.selectable_value(&mut self.active_tab, Tab::AstView, " AST Tree ");
                    ui.selectable_value(&mut self.active_tab, Tab::TokensView, " Tokenizer ");
                    ui.selectable_value(&mut self.active_tab, Tab::GuiBuilder, " GUI Builder ");
                });
                ui.add_space(4.0);

                match self.active_tab {
                    Tab::VmState => {
                        ui.colored_label(retro_gray, "╔═══════════════════════════ VM CPU Watcher ══════════════════════════╗");
                        
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                            .fill(retro_blue)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                ui.columns(2, |columns| {
                                    // Column 1: Disassembly
                                    columns[0].vertical(|ui| {
                                        ui.colored_label(retro_cyan, "┌─ Bytecode Disassembly ────");
                                        ui.add_space(2.0);

                                        if self.disassembly_lines.is_empty() {
                                            ui.colored_label(retro_gray, "  (No bytecode loaded)");
                                        } else {
                                            egui::ScrollArea::vertical().id_source("vm_disasm").show(ui, |ui| {
                                                for &(offset, ref text) in &self.disassembly_lines {
                                                    let is_ip = offset == self.vm.ip;
                                                    if is_ip {
                                                        ui.horizontal(|ui| {
                                                            ui.colored_label(retro_yellow, "⚡");
                                                            ui.colored_label(retro_yellow, format!("{:04} {}", offset, text));
                                                        });
                                                    } else {
                                                        ui.monospace(format!("  {:04} {}", offset, text));
                                                    }
                                                }
                                            });
                                        }
                                    });

                                    // Column 2: Stack & VM Registers
                                    columns[1].vertical(|ui| {
                                        ui.colored_label(retro_cyan, "┌─ Execution Stack ─────────");
                                        ui.add_space(2.0);
                                        
                                        if self.vm.stack.is_empty() {
                                            ui.colored_label(retro_gray, "  (Stack is empty)");
                                        } else {
                                            for (i, val) in self.vm.stack.iter().enumerate().rev() {
                                                ui.monospace(format!("  [{:02}] {:?}", i, val));
                                            }
                                        }
                                        
                                        ui.add_space(10.0);
                                        ui.colored_label(retro_cyan, "┌─ VM Registers ────────────");
                                        ui.monospace(format!("  IP Register:   0x{:04X}", self.vm.ip));
                                        ui.monospace(format!("  Stack Size:    {}", self.vm.stack.len()));
                                        ui.monospace(format!("  GC Heap Size:  {} bytes", self.vm.heap.bytes_allocated));
                                        
                                        if let Some(ref res) = self.execution_result {
                                            ui.add_space(10.0);
                                            match res {
                                                InterpretResult::Ok => {
                                                    ui.colored_label(retro_yellow, "  [VM] Execution OK");
                                                }
                                                InterpretResult::CompileError => {
                                                    ui.colored_label(retro_cyan, "  [VM] Compile Error");
                                                }
                                                InterpretResult::RuntimeError(err) => {
                                                    ui.colored_label(egui::Color32::LIGHT_RED, format!("  [VM] Error: {}", err));
                                                }
                                            }
                                        }
                                    });
                                });
                            });

                        ui.colored_label(retro_gray, "╚═════════════════════════════════════════════════════════════════════╝");
                    }
                    Tab::AstView => {
                        ui.colored_label(retro_gray, "╔═══════════════════════════ AST Structure ═══════════════════════════╗");
                        
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                            .fill(retro_blue)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().id_source("ast_scroll_panel").show(ui, |ui| {
                                    if let Some(ref msg) = self.typecheck_message {
                                        ui.colored_label(retro_yellow, format!("Typechecker Status: {}", msg));
                                        ui.add_space(6.0);
                                    }
                                    
                                    if self.ast_string.is_empty() {
                                        ui.label("(AST is empty)");
                                    } else {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut self.ast_string.clone())
                                                .font(egui::TextStyle::Monospace)
                                                .text_color(retro_white)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(24)
                                                .interactive(false)
                                        );
                                    }
                                });
                            });

                        ui.colored_label(retro_gray, "╚═════════════════════════════════════════════════════════════════════╝");
                    }
                    Tab::TokensView => {
                        ui.colored_label(retro_gray, "╔══════════════════════════ Tokenizer Stream ══════════════════════════╗");
                        
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                            .fill(retro_blue)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().id_source("tokens_scroll_panel").show(ui, |ui| {
                                    if self.tokens_string.is_empty() {
                                        ui.label("(No tokens)");
                                    } else {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut self.tokens_string.clone())
                                                .font(egui::TextStyle::Monospace)
                                                .text_color(retro_white)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(24)
                                                .interactive(false)
                                        );
                                    }
                                });
                            });

                        ui.colored_label(retro_gray, "╚═════════════════════════════════════════════════════════════════════╝");
                    }
                    Tab::GuiBuilder => {
                        ui.colored_label(retro_gray, "╔═══════════════════════════ GUI Builder ═════════════════════════════╗");
                        
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                            .fill(retro_blue)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                ui.columns(2, |columns| {
                                    // Column 1: Toolbox
                                    columns[0].vertical(|ui| {
                                        ui.colored_label(retro_cyan, "┌─ Toolbox ─────────────────");
                                        ui.add_space(4.0);
                                        if ui.button("Add Button").clicked() {
                                            self.gui_components.push(GuiComponent::Button("New Button".to_string()));
                                        }
                                        if ui.button("Add Label").clicked() {
                                            self.gui_components.push(GuiComponent::Label("New Label".to_string()));
                                        }
                                        if ui.button("Add Text Input").clicked() {
                                            self.gui_components.push(GuiComponent::TextInput("Text".to_string()));
                                        }
                                        ui.add_space(8.0);
                                        ui.colored_label(retro_cyan, "┌─ Code Constructs ─────────");
                                        ui.add_space(4.0);
                                        if ui.button("Add Module").clicked() {
                                            self.gui_components.push(GuiComponent::ModuleDef("NewModule".to_string()));
                                        }
                                        if ui.button("Add Class").clicked() {
                                            self.gui_components.push(GuiComponent::ClassDef("NewClass".to_string()));
                                        }
                                        if ui.button("Add Function").clicked() {
                                            self.gui_components.push(GuiComponent::FunctionDef("new_func".to_string()));
                                        }
                                        ui.add_space(8.0);
                                        if ui.button("Clear Canvas").clicked() {
                                            self.gui_components.clear();
                                        }
                                        ui.add_space(8.0);
                                        ui.colored_label(retro_cyan, "┌─ Code Generation ─────────");
                                        ui.add_space(4.0);
                                        if ui.button("Generate Code").clicked() {
                                            let mut generated = String::new();
                                            for comp in &self.gui_components {
                                                match comp {
                                                    GuiComponent::ModuleDef(name) => generated.push_str(&format!("import {};\n\n", name)),
                                                    GuiComponent::ClassDef(name) => generated.push_str(&format!("class {} {{\n    func init() {{\n        // TODO\n    }}\n}}\n\n", name)),
                                                    GuiComponent::FunctionDef(name) => generated.push_str(&format!("func {}() {{\n    // TODO\n}}\n\n", name)),
                                                    GuiComponent::Button(text) => generated.push_str(&format!("// UI Button\nlet btn_{} = \"{}\";\n\n", text.to_lowercase().replace(" ", "_"), text)),
                                                    GuiComponent::Label(text) => generated.push_str(&format!("// UI Label\nlet lbl_{} = \"{}\";\n\n", text.to_lowercase().replace(" ", "_"), text)),
                                                    GuiComponent::TextInput(text) => generated.push_str(&format!("// UI Text Input\nlet input_{} = \"{}\";\n\n", text.to_lowercase().replace(" ", "_"), text)),
                                                }
                                            }
                                            if generated.is_empty() {
                                                generated = "// No components on canvas\n".to_string();
                                            }
                                            self.source_code = generated;
                                            self.current_file_path = "generated_gui.fnx".to_string();
                                            self.compile_code();
                                        }
                                    });

                                    // Column 2: Canvas
                                    columns[1].vertical(|ui| {
                                        ui.colored_label(retro_cyan, "┌─ Canvas ──────────────────");
                                        ui.add_space(4.0);
                                        
                                        egui::Frame::none()
                                            .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                                            .fill(egui::Color32::from_rgb(0, 0, 84)) // Darker blue backing
                                            .inner_margin(8.0)
                                            .show(ui, |ui| {
                                                ui.set_min_height(200.0);
                                                ui.set_min_width(ui.available_width());
                                                
                                                if self.gui_components.is_empty() {
                                                    ui.colored_label(retro_gray, "Drop widgets here");
                                                } else {
                                                    let mut to_remove = None;
                                                    for (i, comp) in self.gui_components.iter_mut().enumerate() {
                                                        ui.horizontal(|ui| {
                                                            match comp {
                                                                GuiComponent::Button(text) => {
                                                                    let _ = ui.button(text.clone());
                                                                    ui.add(egui::TextEdit::singleline(text).text_color(retro_white));
                                                                }
                                                                GuiComponent::Label(text) => {
                                                                    ui.label(text.clone());
                                                                    ui.add(egui::TextEdit::singleline(text).text_color(retro_white));
                                                                }
                                                                GuiComponent::TextInput(text) => {
                                                                    let mut fake_input = text.clone();
                                                                    ui.add(egui::TextEdit::singleline(&mut fake_input).text_color(retro_white));
                                                                    ui.add(egui::TextEdit::singleline(text).text_color(retro_white));
                                                                }
                                                                GuiComponent::ClassDef(name) => {
                                                                    egui::Frame::none()
                                                                        .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                                                                        .inner_margin(4.0)
                                                                        .show(ui, |ui| {
                                                                            ui.colored_label(retro_yellow, "Class");
                                                                            ui.add(egui::TextEdit::singleline(name).text_color(retro_white));
                                                                        });
                                                                }
                                                                GuiComponent::FunctionDef(name) => {
                                                                    egui::Frame::none()
                                                                        .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                                                                        .inner_margin(4.0)
                                                                        .show(ui, |ui| {
                                                                            ui.colored_label(retro_yellow, "Func ");
                                                                            ui.add(egui::TextEdit::singleline(name).text_color(retro_white));
                                                                        });
                                                                }
                                                                GuiComponent::ModuleDef(name) => {
                                                                    egui::Frame::none()
                                                                        .stroke(egui::Stroke::new(1.0_f32, retro_gray))
                                                                        .inner_margin(4.0)
                                                                        .show(ui, |ui| {
                                                                            ui.colored_label(retro_yellow, "Mod  ");
                                                                            ui.add(egui::TextEdit::singleline(name).text_color(retro_white));
                                                                        });
                                                                }
                                                            }
                                                            if ui.button(" X ").clicked() {
                                                                to_remove = Some(i);
                                                            }
                                                        });
                                                        ui.add_space(2.0);
                                                    }
                                                    if let Some(i) = to_remove {
                                                        self.gui_components.remove(i);
                                                    }
                                                }
                                            });
                                    });
                                });
                            });

                        ui.colored_label(retro_gray, "╚═════════════════════════════════════════════════════════════════════╝");
                    }
                }

                // Global error panel
                if let Some(ref err) = self.error_message {
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(170, 0, 0)) // Red background
                        .inner_margin(6.0)
                        .rounding(0.0)
                        .show(ui, |ui| {
                            ui.colored_label(retro_white, err);
                        });
                }
            });

        // --- F1 POPUP HELP WINDOW ---
        if self.show_help {
            let mut close_help = false;
            egui::Window::new(" Help Screen ")
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::default().fill(retro_gray).stroke(egui::Stroke::new(2.0_f32, retro_yellow)).inner_margin(12.0))
                .show(ctx, |ui| {
                    ui.style_mut().visuals.override_text_color = Some(retro_black);
                    
                    ui.monospace("┌───────────────────────────────────┐");
                    ui.monospace("│ Turbo Finix IDE Keyboard Commands │");
                    ui.monospace("├───────────────────────────────────┤");
                    ui.monospace("│  F1        - Show Help            │");
                    ui.monospace("│  F2        - Save File            │");
                    ui.monospace("│  F3        - Load Demo Template   │");
                    ui.monospace("│  F5        - Run VM bytecode      │");
                    ui.monospace("│  F8        - Single Step VM Inst  │");
                    ui.monospace("│  F9        - Compile Code         │");
                    ui.monospace("│  F10       - Reset VM state       │");
                    ui.monospace("│  Alt+X     - Exit (Close window)  │");
                    ui.monospace("└───────────────────────────────────┘");
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = retro_cyan;
                        ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, retro_white);
                        if ui.button("  OK  ").clicked() {
                            close_help = true;
                        }
                    });
                });
            if close_help || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_help = false;
            }
        }

        // --- SAVE DIALOG ---
        if self.show_save_dialog {
            let mut close_dialog = false;
            egui::Window::new(" Save File As ")
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::default().fill(retro_gray).stroke(egui::Stroke::new(2.0_f32, retro_yellow)).inner_margin(12.0))
                .show(ctx, |ui| {
                    ui.style_mut().visuals.override_text_color = Some(retro_black);
                    ui.label("Enter file name to save:");
                    ui.add(egui::TextEdit::singleline(&mut self.dialog_input_path).text_color(retro_white));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = retro_cyan;
                        ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, retro_white);
                        if ui.button("  Save  ").clicked() {
                            self.current_file_path = self.dialog_input_path.clone();
                            let _ = std::fs::write(&self.current_file_path, &self.source_code);
                            close_dialog = true;
                        }
                        if ui.button(" Cancel ").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            if close_dialog || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_save_dialog = false;
            }
        }

        // --- OPEN DIALOG ---
        if self.show_open_dialog {
            let mut close_dialog = false;
            egui::Window::new(" Open File ")
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::default().fill(retro_gray).stroke(egui::Stroke::new(2.0_f32, retro_yellow)).inner_margin(12.0))
                .show(ctx, |ui| {
                    ui.style_mut().visuals.override_text_color = Some(retro_black);
                    ui.label("Enter file name to open:");
                    ui.add(egui::TextEdit::singleline(&mut self.dialog_input_path).text_color(retro_white));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = retro_cyan;
                        ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, retro_white);
                        if ui.button("  Open  ").clicked() {
                            if let Ok(content) = std::fs::read_to_string(&self.dialog_input_path) {
                                self.source_code = content;
                                self.current_file_path = self.dialog_input_path.clone();
                                self.compile_code();
                            }
                            close_dialog = true;
                        }
                        if ui.button(" Cancel ").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            if close_dialog || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_open_dialog = false;
            }
        }

        // --- NEW PROJECT DIALOG ---
        if self.show_new_project_dialog {
            let mut close_dialog = false;
            egui::Window::new(" Create New Project ")
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::default().fill(retro_gray).stroke(egui::Stroke::new(2.0_f32, retro_yellow)).inner_margin(12.0))
                .show(ctx, |ui| {
                    ui.style_mut().visuals.override_text_color = Some(retro_black);
                    ui.label("Enter new project directory name:");
                    ui.add(egui::TextEdit::singleline(&mut self.dialog_input_path).text_color(retro_white));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = retro_cyan;
                        ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, retro_white);
                        if ui.button(" Create ").clicked() {
                            let _ = std::fs::create_dir_all(&self.dialog_input_path);
                            self.current_file_path = format!("{}/main.fnx", self.dialog_input_path);
                            self.source_code = "println(\"Welcome to your new project!\");\n".to_string();
                            let _ = std::fs::write(&self.current_file_path, &self.source_code);
                            self.compile_code();
                            close_dialog = true;
                        }
                        if ui.button(" Cancel ").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            if close_dialog || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_new_project_dialog = false;
            }
        }
    }
}

fn disassemble_chunk_to_lines(chunk: &Chunk) -> Vec<(usize, String)> {
    let mut lines = Vec::new();
    let mut offset = 0;
    while offset < chunk.code.len() {
        let current_offset = offset;
        let instr = chunk.code[offset];
        let (text, next_offset) = match instr {
            OP_RETURN => ("OP_RETURN".to_string(), offset + 1),
            OP_POP => ("OP_POP".to_string(), offset + 1),
            OP_NULL => ("OP_NULL".to_string(), offset + 1),
            OP_TRUE => ("OP_TRUE".to_string(), offset + 1),
            OP_FALSE => ("OP_FALSE".to_string(), offset + 1),
            OP_ADD => ("OP_ADD".to_string(), offset + 1),
            OP_SUB => ("OP_SUB".to_string(), offset + 1),
            OP_MUL => ("OP_MUL".to_string(), offset + 1),
            OP_DIV => ("OP_DIV".to_string(), offset + 1),
            OP_NEGATE => ("OP_NEGATE".to_string(), offset + 1),
            OP_NOT => ("OP_NOT".to_string(), offset + 1),
            OP_EQUAL => ("OP_EQUAL".to_string(), offset + 1),
            OP_GREATER => ("OP_GREATER".to_string(), offset + 1),
            OP_LESS => ("OP_LESS".to_string(), offset + 1),
            OP_PRINT => ("OP_PRINT".to_string(), offset + 1),
            OP_PRINTLN => ("OP_PRINTLN".to_string(), offset + 1),
            OP_CONSTANT => {
                if offset + 1 < chunk.code.len() {
                    let idx = chunk.code[offset + 1] as usize;
                    if idx < chunk.constants.len() {
                        let val = &chunk.constants[idx];
                        (format!("OP_CONSTANT {} ('{}')", idx, val), offset + 2)
                    } else {
                        (format!("OP_CONSTANT {} [INVALID INDEX]", idx), offset + 2)
                    }
                } else {
                    ("OP_CONSTANT [TRUNCATED]".to_string(), offset + 1)
                }
            }
            OP_GET_LOCAL => {
                if offset + 1 < chunk.code.len() {
                    let slot = chunk.code[offset + 1];
                    (format!("OP_GET_LOCAL {}", slot), offset + 2)
                } else {
                    ("OP_GET_LOCAL [TRUNCATED]".to_string(), offset + 1)
                }
            }
            OP_SET_LOCAL => {
                if offset + 1 < chunk.code.len() {
                    let slot = chunk.code[offset + 1];
                    (format!("OP_SET_LOCAL {}", slot), offset + 2)
                } else {
                    ("OP_SET_LOCAL [TRUNCATED]".to_string(), offset + 1)
                }
            }
            OP_JUMP => {
                if offset + 2 < chunk.code.len() {
                    let b1 = chunk.code[offset + 1] as u16;
                    let b2 = chunk.code[offset + 2] as u16;
                    let jump = (b1 << 8) | b2;
                    (format!("OP_JUMP {} -> offset {}", jump, offset + 3 + jump as usize), offset + 3)
                } else {
                    ("OP_JUMP [TRUNCATED]".to_string(), offset + 1)
                }
            }
            OP_JUMP_IF_FALSE => {
                if offset + 2 < chunk.code.len() {
                    let b1 = chunk.code[offset + 1] as u16;
                    let b2 = chunk.code[offset + 2] as u16;
                    let jump = (b1 << 8) | b2;
                    (format!("OP_JUMP_IF_FALSE {} -> offset {}", jump, offset + 3 + jump as usize), offset + 3)
                } else {
                    ("OP_JUMP_IF_FALSE [TRUNCATED]".to_string(), offset + 1)
                }
            }
            OP_LOOP => {
                if offset + 2 < chunk.code.len() {
                    let b1 = chunk.code[offset + 1] as u16;
                    let b2 = chunk.code[offset + 2] as u16;
                    let jump = (b1 << 8) | b2;
                    (format!("OP_LOOP {} -> offset {}", jump, offset + 3 - jump as usize), offset + 3)
                } else {
                    ("OP_LOOP [TRUNCATED]".to_string(), offset + 1)
                }
            }
            other => (format!("UNKNOWN OPCODE {}", other), offset + 1),
        };
        lines.push((current_offset, text));
        offset = next_offset;
    }
    lines
}
