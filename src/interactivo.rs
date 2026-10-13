//! `GM_POLIGONO` interactivo: clic en el centro, mover el cursor (el host
//! previsualiza el polígono) y clic para fijar el radio.

use ocs_plugin_api::host::{CommandStep, InteractiveCommand, PreviewWire};

use crate::dispatch::{figura, LAYER_FIGURAS};
use crate::state::state;

pub struct PoligonoInteractivo {
    lados: usize,
    centro: Option<[f64; 3]>,
}

impl PoligonoInteractivo {
    pub fn new(lados: usize) -> Self {
        Self { lados, centro: None }
    }

    fn vertices(&self, c: [f64; 3], pt: [f64; 3]) -> Option<Vec<geometria::Punto2>> {
        let radio = (pt[0] - c[0]).hypot(pt[1] - c[1]);
        geometria::poligono_regular(self.lados, radio, (c[0], c[1]))
    }
}

impl InteractiveCommand for PoligonoInteractivo {
    fn prompt(&self) -> String {
        match self.centro {
            None => format!("GM_POLIGONO ({} lados): centro", self.lados),
            Some(_) => "GM_POLIGONO: radio (punto en la circunferencia)".to_string(),
        }
    }

    fn on_point(&mut self, pt: [f64; 3]) -> CommandStep {
        let Some(c) = self.centro else {
            self.centro = Some(pt);
            return CommandStep::NeedPoint;
        };
        match self.vertices(c, pt) {
            Some(v) => {
                let nombre = state().nueva_figura();
                CommandStep::CommitAndEnd(figura(&v, LAYER_FIGURAS, &nombre, "POLIGONO"))
            }
            // Radio cero: pedir otro punto.
            None => CommandStep::NeedPoint,
        }
    }

    fn on_cursor_move(&mut self, pt: [f64; 3]) -> Vec<PreviewWire> {
        let Some(c) = self.centro else {
            return Vec::new();
        };
        let Some(v) = self.vertices(c, pt) else {
            return Vec::new();
        };
        let mut contorno: Vec<[f64; 3]> = v.iter().map(|&(x, y)| [x, y, 0.0]).collect();
        contorno.push(contorno[0]);
        vec![PreviewWire::new(contorno), PreviewWire::line(c, pt, Some([1.0, 0.8, 0.2, 1.0]))]
    }
}
