use enclose::enclose;
use gtk::prelude::*;
use j2gbc::debug::Register8;

use crate::SystemRef;

const DEBUGGER_UI: &str = include_str!("../../assets/ui/debugger.glade");

#[derive(Clone)]
struct Context {
    system: SystemRef,
    pause_button: gtk::ToolButton,
    resume_button: gtk::ToolButton,
    step_button: gtk::ToolButton,

    register_af: gtk::Label,
    register_bc: gtk::Label,
    register_de: gtk::Label,
    register_sp: gtk::Label,
    register_pc: gtk::Label,
}

pub fn load_debugger(system: &SystemRef) -> gtk::Window {
    let builder = gtk::Builder::new_from_string(DEBUGGER_UI);
    let window: gtk::Window = builder.get_object("debugger_window").unwrap();
    let context = Context::from_builder(system.clone(), builder);
    context.running();

    context
        .pause_button
        .connect_clicked(enclose!((context) move |_| {
            context.system.borrow_mut().debugger().pause();
            context.halted();
        }));
    context
        .resume_button
        .connect_clicked(enclose!((context) move |_| {
            context.system.borrow_mut().debugger().resume();
            context.running();
        }));
    context
        .step_button
        .connect_clicked(enclose!((context) move |_| {
            context.system.borrow_mut().debugger().step();
            context.halted();
        }));

    window.show_all();
    window
}

impl Context {
    pub fn from_builder(system: SystemRef, builder: gtk::Builder) -> Context {
        Context {
            system,
            pause_button: builder.get_object("pause_button").unwrap(),
            resume_button: builder.get_object("resume_button").unwrap(),
            step_button: builder.get_object("step_button").unwrap(),

            register_af: builder.get_object("register_AF").unwrap(),
            register_bc: builder.get_object("register_BC").unwrap(),
            register_de: builder.get_object("register_DE").unwrap(),
            register_sp: builder.get_object("register_SP").unwrap(),
            register_pc: builder.get_object("register_PC").unwrap(),
        }
    }

    pub fn running(&self) {
        self.resume_button.set_sensitive(false);
        self.step_button.set_sensitive(false);
        self.pause_button.set_sensitive(true);
    }

    pub fn halted(&self) {
        self.resume_button.set_sensitive(true);
        self.step_button.set_sensitive(true);
        self.pause_button.set_sensitive(false);

        self.update_regs();
    }

    pub fn update_regs(&self) {
        let mut sys = self.system.borrow_mut();
        let debug = sys.debugger();
        self.register_af.set_text(
            format!(
                "0x{:02x}{:02x}",
                debug.read_reg(Register8::A),
                debug.read_reg(Register8::F)
            )
            .as_str(),
        );
        self.register_bc.set_text(
            format!(
                "0x{:02x}{:02x}",
                debug.read_reg(Register8::B),
                debug.read_reg(Register8::C)
            )
            .as_str(),
        );
        self.register_de.set_text(
            format!(
                "0x{:02x}{:02x}",
                debug.read_reg(Register8::D),
                debug.read_reg(Register8::E)
            )
            .as_str(),
        );

        self.register_pc
            .set_text(format!("{}", debug.read_pc(),).as_str());
        self.register_sp
            .set_text(format!("{}", debug.read_sp(),).as_str());
    }
}
