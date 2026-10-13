//! Geometría — plugin de ejemplo externo para Open CAD Studio, compilado como
//! `cdylib` que el host carga en tiempo de ejecución.
//!
//! Sigue la misma arquitectura que el plugin Land Survey: depende solo de
//! `ocs_plugin_api` (feature `host`) y del codec CAD del host, nunca del binario
//! de OpenCADStudio. Expone un [`PluginManifest`], una pestaña del ribbon
//! ([`ribbon`]), el punto de entrada [`BuiltinPlugin`] y la exportación C-ABI
//! `export_plugin!`. La matemática vive en el crate `geometria`
//! (`crates/geometria/`); [`dispatch`] la convierte en comandos, entidades y XDATA.

use ocs_plugin_api::host::{BuiltinPlugin, HostApi};
use ocs_plugin_api::manifest::{ApiVersion, PluginManifest};
use ocs_plugin_api::ribbon::CadModule;

mod dispatch;
mod interactivo;
mod ribbon;
mod state;

/// Id del plugin en notación DNS inversa; clave de la carpeta de plugins.
pub const PLUGIN_ID: &str = "opencad.geometria";

// Mantener sincronizado con `plugin.toml` (la versión viene de Cargo.toml).
static MANIFEST: PluginManifest = PluginManifest {
    id: PLUGIN_ID,
    name: "Geometría",
    version: env!("CARGO_PKG_VERSION"),
    description: "Plugin de ejemplo: polígonos regulares, estrellas, rejillas e importación de puntos CSV.",
    api_version: ApiVersion::CURRENT,
    ribbon_order: 60,
    // Aplicaciones XDATA que escribe el plugin; el host registra los APPID para
    // que sobrevivan a un guardado DWG/DXF. Reflejado en `xdata_apps` de plugin.toml.
    xdata_apps: &[dispatch::XDATA_FIGURA, dispatch::XDATA_PUNTO],
    command_prefixes: &["GM_"],
};

/// Punto de entrada que se entrega al host.
struct GeometriaPlugin;

impl BuiltinPlugin for GeometriaPlugin {
    fn manifest(&self) -> &'static PluginManifest {
        &MANIFEST
    }

    fn ribbon(&self) -> Box<dyn CadModule> {
        Box::new(ribbon::GeometriaModule)
    }

    fn dispatch(&self, host: &mut dyn HostApi, cmd: &str) -> bool {
        dispatch::handle(host, cmd)
    }
}

// Emite los símbolos C-ABI que busca el cargador del host.
ocs_plugin_api::export_plugin!(GeometriaPlugin);
