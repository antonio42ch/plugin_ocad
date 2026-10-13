//! Enrutado de comandos del plugin Geometría. Todos los comandos `GM_` llegan
//! aquí vía `BuiltinPlugin::dispatch`. La matemática está en el crate
//! `geometria`; este archivo la convierte en entidades del codec + XDATA.

use std::fs;

use codec::xdata::{ExtendedDataRecord, XDataValue};
use codec::{EntityType, Line, LwPolyline, Point as CadPoint, Text, Vector2, Vector3};
use ocs_plugin_api::host::HostApi;

use crate::interactivo::PoligonoInteractivo;
use crate::state::state;

/// XDATA en cada figura cerrada: `[String(nombre), String(tipo)]`
/// (tipo: `POLIGONO` / `ESTRELLA`).
pub const XDATA_FIGURA: &str = "GEOMETRIA_FIGURA";

/// XDATA en cada punto importado: `[String(nombre), String(descripcion)]`.
pub const XDATA_PUNTO: &str = "GEOMETRIA_PUNTO";

pub const LAYER_FIGURAS: &str = "GM-FIGURAS";
const LAYER_REJILLA: &str = "GM-REJILLA";
const LAYER_PUNTOS: &str = "GM-PUNTOS";
const LAYER_ETIQUETAS: &str = "GM-ETIQUETAS";

pub fn handle(host: &mut dyn HostApi, cmd: &str) -> bool {
    // Se enruta por la primera palabra; `cmd` se conserva entero para los
    // argumentos (las rutas distinguen mayúsculas y pueden tener espacios).
    let verbo = cmd.split_whitespace().next().unwrap_or("").to_uppercase();
    if !verbo.starts_with("GM_") {
        return false;
    }
    let args: Vec<&str> = cmd.split_whitespace().skip(1).collect();
    match verbo.as_str() {
        "GM_HOLA" => host.push_info(
            "¡Hola desde el plugin Geometría! Comandos: GM_POLIGONO, GM_LADOS, \
             GM_ESTRELLA, GM_REJILLA, GM_PUNTOS, GM_LISTAR. Ver PLUGIN.md.",
        ),
        "GM_POLIGONO" => poligono(host, &args),
        "GM_LADOS" => lados(host, &args),
        "GM_ESTRELLA" => estrella(host, &args),
        "GM_REJILLA" => rejilla(host, &args),
        "GM_PUNTOS" => importar_puntos(host, primer_argumento(cmd)),
        "GM_LISTAR" => listar(host),
        _ => return false,
    }
    true
}

/// Todo lo que va tras el verbo, recortado (para rutas con espacios).
fn primer_argumento(cmd: &str) -> &str {
    cmd.splitn(2, char::is_whitespace).nth(1).map(str::trim).unwrap_or("")
}

/// Lee `args` como números; `None` si alguno no lo es.
fn numeros(args: &[&str]) -> Option<Vec<f64>> {
    args.iter().map(|a| a.parse::<f64>().ok().filter(|v| v.is_finite())).collect()
}

/// Centro opcional en las posiciones `i`, `i+1`; por defecto el origen.
fn centro(n: &[f64], i: usize) -> (f64, f64) {
    (n.get(i).copied().unwrap_or(0.0), n.get(i + 1).copied().unwrap_or(0.0))
}

/// Construye una LwPolyline cerrada en `layer`, con su registro `GEOMETRIA_FIGURA`
/// ya puesto en la entidad (sirve tanto para `add_entity` como para
/// `CommandStep::Commit` del modo interactivo).
pub fn figura(vertices: &[geometria::Punto2], layer: &str, nombre: &str, tipo: &str) -> EntityType {
    let mut lw = LwPolyline::new();
    for &(x, y) in vertices {
        lw.add_point_with_bulge(Vector2::new(x, y), 0.0);
    }
    lw.is_closed = true;
    let mut ent = EntityType::LwPolyline(lw);
    ent.common_mut().layer = layer.to_string();
    ent.common_mut().extended_data.add_record(registro(XDATA_FIGURA, nombre, tipo));
    ent
}

fn registro(app: &str, a: &str, b: &str) -> ExtendedDataRecord {
    let mut rec = ExtendedDataRecord::new(app);
    rec.add_value(XDataValue::String(a.to_string()));
    rec.add_value(XDataValue::String(b.to_string()));
    rec
}

/// Añade una entidad en `layer`.
fn en_capa(host: &mut dyn HostApi, mut ent: EntityType, layer: &str) -> codec::Handle {
    ent.common_mut().layer = layer.to_string();
    host.add_entity(ent)
}

/// Añade una figura, la etiqueta con su nombre en el centro y avisa.
fn dibujar_figura(host: &mut dyn HostApi, v: &[geometria::Punto2], c: (f64, f64), tipo: &str) {
    let nombre = state().nueva_figura();
    host.push_undo(&format!("GM_{tipo}"));
    let handle = host.add_entity(figura(v, LAYER_FIGURAS, &nombre, tipo));
    // `write_record` registra el APPID en el documento (round-trip DWG/DXF).
    host.write_record(handle, registro(XDATA_FIGURA, &nombre, tipo));
    let alto = (geometria::perimetro(v) / 40.0).max(0.1);
    let etiqueta = Text::with_value(nombre.clone(), Vector3::new(c.0, c.1, 0.0)).with_height(alto);
    en_capa(host, EntityType::Text(etiqueta), LAYER_ETIQUETAS);
    host.set_dirty();
    host.push_output(&format!(
        "{nombre}: {tipo} de {} vértices, área {:.3}, perímetro {:.3}",
        v.len(),
        geometria::area(v),
        geometria::perimetro(v)
    ));
}

/// `GM_POLIGONO [<lados> <radio> [cx cy]]` — sin argumentos, modo interactivo.
fn poligono(host: &mut dyn HostApi, args: &[&str]) {
    if args.is_empty() {
        let lados = state().lados;
        host.start_interactive(Box::new(PoligonoInteractivo::new(lados)));
        return;
    }
    const USO: &str = "Uso: GM_POLIGONO <lados> <radio> [cx cy]  (sin argumentos: interactivo)";
    let Some(n) = numeros(args).filter(|n| n.len() == 2 || n.len() == 4) else {
        return host.push_info(USO);
    };
    let c = centro(&n, 2);
    match geometria::poligono_regular(n[0] as usize, n[1], c) {
        Some(v) => dibujar_figura(host, &v, c, "POLIGONO"),
        None => host.push_error("GM_POLIGONO: hacen falta al menos 3 lados y un radio > 0."),
    }
}

/// `GM_LADOS [<n>]` — lados del polígono interactivo; sin argumento, los muestra.
fn lados(host: &mut dyn HostApi, args: &[&str]) {
    let Some(arg) = args.first() else {
        let n = state().lados;
        return host.push_info(&format!("GM_LADOS: el polígono interactivo usa {n} lados."));
    };
    match arg.parse::<usize>() {
        Ok(n) if n >= 3 => {
            state().lados = n;
            host.push_info(&format!("GM_LADOS: ahora {n} lados."));
        }
        _ => host.push_error("GM_LADOS: indica un número entero ≥ 3."),
    }
}

/// `GM_ESTRELLA <puntas> <radio_ext> <radio_int> [cx cy]`
fn estrella(host: &mut dyn HostApi, args: &[&str]) {
    const USO: &str = "Uso: GM_ESTRELLA <puntas> <radio_ext> <radio_int> [cx cy]";
    let Some(n) = numeros(args).filter(|n| n.len() == 3 || n.len() == 5) else {
        return host.push_info(USO);
    };
    let c = centro(&n, 3);
    match geometria::estrella(n[0] as usize, n[1], n[2], c) {
        Some(v) => dibujar_figura(host, &v, c, "ESTRELLA"),
        None => host.push_error("GM_ESTRELLA: puntas ≥ 2 y 0 < radio_int < radio_ext."),
    }
}

/// `GM_REJILLA <nx> <ny> <paso> [x0 y0]` — rejilla de líneas.
fn rejilla(host: &mut dyn HostApi, args: &[&str]) {
    const USO: &str = "Uso: GM_REJILLA <nx> <ny> <paso> [x0 y0]";
    let Some(n) = numeros(args).filter(|n| n.len() == 3 || n.len() == 5) else {
        return host.push_info(USO);
    };
    let Some(segmentos) = geometria::rejilla(n[0] as usize, n[1] as usize, n[2], centro(&n, 3)) else {
        return host.push_error("GM_REJILLA: nx, ny ≥ 1 y paso > 0.");
    };
    host.push_undo("GM_REJILLA");
    // Un solo viaje IPC para todas las líneas (ver batch.rs en Land Survey).
    let lineas: Vec<EntityType> = segmentos
        .iter()
        .map(|&((x1, y1), (x2, y2))| {
            let mut e = EntityType::Line(Line::from_points(Vector3::new(x1, y1, 0.0), Vector3::new(x2, y2, 0.0)));
            e.common_mut().layer = LAYER_REJILLA.to_string();
            e
        })
        .collect();
    let total = host.add_entities(lineas).len();
    host.set_dirty();
    host.push_output(&format!("GM_REJILLA: {total} líneas en {LAYER_REJILLA}."));
}

/// `GM_PUNTOS <ruta>` — importa un CSV `nombre,x,y[,z][,desc]` como entidades
/// `Point` etiquetadas y marcadas con `GEOMETRIA_PUNTO`.
fn importar_puntos(host: &mut dyn HostApi, ruta: &str) {
    if ruta.is_empty() {
        return host.push_info("Uso: GM_PUNTOS <ruta.csv>  (columnas nombre,x,y[,z][,descripcion])");
    }
    let texto = match fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(e) => return host.push_error(&format!("GM_PUNTOS: no se pudo leer {ruta}: {e}")),
    };
    let lectura = geometria::leer_csv(&texto);
    if lectura.puntos.is_empty() {
        return host.push_error("GM_PUNTOS: el archivo no contiene puntos válidos.");
    }
    host.push_undo("GM_PUNTOS");
    for p in &lectura.puntos {
        let punto = EntityType::Point(CadPoint::at(Vector3::new(p.x, p.y, p.z)));
        let h = en_capa(host, punto, LAYER_PUNTOS);
        host.write_record(h, registro(XDATA_PUNTO, &p.nombre, &p.descripcion));
        let etiqueta = Text::with_value(p.nombre.clone(), Vector3::new(p.x + 0.5, p.y + 0.5, 0.0)).with_height(1.0);
        en_capa(host, EntityType::Text(etiqueta), LAYER_ETIQUETAS);
    }
    host.set_dirty();
    host.push_output(&format!("GM_PUNTOS: {} punto(s) importados en {LAYER_PUNTOS}.", lectura.puntos.len()));
    if !lectura.omitidas.is_empty() {
        host.push_error(&format!("GM_PUNTOS: líneas omitidas: {:?}", lectura.omitidas));
    }
}

/// Las cadenas de un registro XDATA de `app` en una entidad.
fn cadenas(e: &EntityType, app: &str) -> Option<Vec<String>> {
    let rec = e.common().extended_data.get_record(app)?;
    Some(
        rec.values
            .iter()
            .filter_map(|v| match v {
                XDataValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
    )
}

/// `GM_LISTAR` — relee del dibujo las figuras y puntos del plugin a partir de
/// su XDATA y su geometría actual (si el usuario las editó, los datos cambian).
fn listar(host: &mut dyn HostApi) {
    // Primero se recoge (préstamo inmutable del documento) y luego se escribe (&mut).
    let mut filas = Vec::new();
    for e in host.document().entities() {
        if let Some(s) = cadenas(e, XDATA_FIGURA) {
            if let EntityType::LwPolyline(lw) = e {
                let v: Vec<_> = lw.vertices.iter().map(|v| (v.location.x, v.location.y)).collect();
                filas.push(format!(
                    "  {:<6} {:<9} área {:>12.3}  perímetro {:>10.3}",
                    s.first().map_or("", |x| x),
                    s.get(1).map_or("", |x| x),
                    geometria::area(&v),
                    geometria::perimetro(&v)
                ));
            }
        } else if let Some(s) = cadenas(e, XDATA_PUNTO) {
            if let EntityType::Point(p) = e {
                filas.push(format!(
                    "  {:<6} PUNTO     X {:.3}  Y {:.3}  Z {:.3}  {}",
                    s.first().map_or("", |x| x),
                    p.location.x,
                    p.location.y,
                    p.location.z,
                    s.get(1).map_or("", |x| x)
                ));
            }
        }
    }
    if filas.is_empty() {
        return host.push_output("GM_LISTAR: no hay objetos del plugin Geometría en el dibujo.");
    }
    host.push_output(&format!("GM_LISTAR: {} objeto(s):", filas.len()));
    for f in &filas {
        host.push_output(f);
    }
}
