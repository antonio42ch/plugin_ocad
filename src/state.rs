//! Estado local del proceso del plugin. Desde OCS 0.7.x cada plugin corre en su
//! propio proceso "runner", y `ensure_plugin_state` del host no funciona fuera
//! de proceso (un `dyn Any` no cruza el IPC). Un global del proceso es el
//! reemplazo recomendado: el runner es por plugin, así que el estado queda
//! limitado a este plugin (compartido entre pestañas).

use std::sync::{Mutex, MutexGuard, OnceLock};

static STATE: OnceLock<Mutex<GeometriaState>> = OnceLock::new();

/// Bloquea el estado del plugin. No llamar mientras otra guarda de la misma
/// cadena de llamadas siga viva: `Mutex` no es reentrante.
pub fn state() -> MutexGuard<'static, GeometriaState> {
    STATE
        .get_or_init(|| Mutex::new(GeometriaState::default()))
        .lock()
        .unwrap_or_else(|envenenado| envenenado.into_inner())
}

pub struct GeometriaState {
    /// Lados que usa `GM_POLIGONO` en modo interactivo (`GM_LADOS <n>`).
    pub lados: usize,
    /// Figuras dibujadas en esta sesión, para numerarlas (`F1`, `F2`, …).
    pub figuras: usize,
}

impl Default for GeometriaState {
    fn default() -> Self {
        Self { lados: 6, figuras: 0 }
    }
}

impl GeometriaState {
    /// Siguiente nombre de figura.
    pub fn nueva_figura(&mut self) -> String {
        self.figuras += 1;
        format!("F{}", self.figuras)
    }
}
