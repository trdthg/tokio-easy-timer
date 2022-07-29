#![allow(unused)]

pub struct Ui {}

pub trait Component<Params, Content> {
    fn call(&self, ui: &mut Ui, params: Params, content: Content);
}

impl<F, P1, Content> Component<(P1,), Content> for F
where
    P1: PartialEq + Clone + 'static,
    Content: FnOnce(&mut Ui),
    F: Fn(&mut Ui, P1, Content),
{
    fn call(&self, ui: &mut Ui, params: (P1,), content: Content) {
        let (p1,) = params;
        self(ui, p1, content)
    }
}

impl<F, P1, P2, Content> Component<(P1, P2), Content> for F
where
    P1: PartialEq + Clone + 'static,
    P2: PartialEq + Clone + 'static,
    Content: FnOnce(&mut Ui),
    F: Fn(&mut Ui, P1, P2, Content),
{
    fn call(&self, ui: &mut Ui, params: (P1, P2), content: Content) {
        let (p1, p2) = params;
        self(ui, p1, p2, content)
    }
}

impl<F, P1, P2, P3, Content> Component<(P1, P2, P3), Content> for F
where
    P1: PartialEq + Clone + 'static,
    P2: PartialEq + Clone + 'static,
    Content: FnOnce(&mut Ui),
    F: Fn(&mut Ui, P1, P2, Content),
{
    fn call(&self, ui: &mut Ui, params: (P1, P2, P3), content: Content) {
        let (p1, p2, p3) = params;
        self(ui, p1, p2, content)
    }
}

pub fn memoize<
    Params: PartialEq + Clone + 'static,
    C: FnOnce(&mut Ui),
    Comp: Component<Params, C>,
>(
    ui: &mut Ui,
    component: Comp,
    params: Params,
    content: C,
) {
    component.call(ui, params, content);
}

fn comp2(ui: &mut Ui, a: u8, b: u32, f: impl FnOnce(&mut Ui)) {
    f(ui);
}

fn main() {
    let mut ui = Ui {};
    memoize(&mut ui, comp2, (2, 3), |_| {});
}
