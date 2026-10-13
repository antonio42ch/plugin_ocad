//! La pestaña "Geometría" del ribbon. Solo datos: el host los dibuja.

use ocs_plugin_api::ribbon::{CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, ToolDef};

pub struct GeometriaModule;

/// Botón que despacha `cmd` tal cual al pulsarlo.
fn boton(id: &'static str, label: &'static str, glifo: &'static str) -> RibbonItem {
    RibbonItem::LargeTool(ToolDef {
        id,
        label,
        icon: IconKind::Glyph(glifo),
        event: ModuleEvent::Command(id.to_string()),
    })
}

impl CadModule for GeometriaModule {
    fn id(&self) -> &'static str {
        "geometria"
    }

    fn title(&self) -> &'static str {
        "Geometría"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        // Contrato del host: devolver un slice prestado; se construye una vez
        // y se cachea (patrón OnceLock).
        static GROUPS: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        GROUPS.get_or_init(|| {
            vec![
                RibbonGroup {
                    title: "Dibujar",
                    tools: vec![
                        // Sin argumentos inicia el modo interactivo (clic centro + clic radio).
                        boton("GM_POLIGONO", "Polígono", "\u{2B21}"), // ⬡
                        // Imprime el uso; el usuario escribe los parámetros.
                        boton("GM_ESTRELLA", "Estrella", "\u{2605}"), // ★
                        boton("GM_REJILLA", "Rejilla", "\u{25A6}"),   // ▦
                    ],
                },
                RibbonGroup {
                    title: "Puntos",
                    tools: vec![
                        // El host abre un selector de archivos nativo y despacha
                        // "GM_PUNTOS <ruta>" con la ruta tal cual.
                        RibbonItem::LargeTool(ToolDef {
                            id: "GM_PUNTOS",
                            label: "Importar CSV",
                            icon: IconKind::Glyph("\u{2295}"), // ⊕
                            event: ModuleEvent::PluginFileDialog {
                                command: "GM_PUNTOS".to_string(),
                                title: "Importar puntos CSV".to_string(),
                                filter_name: "Puntos (nombre,x,y[,z][,desc])".to_string(),
                                extensions: vec!["csv".to_string(), "txt".to_string()],
                            },
                        }),
                    ],
                },
                RibbonGroup {
                    title: "Consultar",
                    tools: vec![
                        boton("GM_LISTAR", "Listar", "\u{2261}"), // ≡
                        boton("GM_HOLA", "Ayuda", "?"),
                    ],
                },
            ]
        })
    }
}
