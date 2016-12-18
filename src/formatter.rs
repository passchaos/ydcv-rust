use ansi_term::Colour;

pub trait YDCVFormatter {
    fn head_color(&self) -> Colour {
        Colour::RGB(26, 159, 160)
    }

    fn phonetic_color(&self) -> Colour {
        Colour::RGB(220, 186, 40)
    }

    fn reference_color(&self) -> Colour {
        Colour::RGB(138, 88, 164)
    }

    fn translation_description(&self) -> String;
}
