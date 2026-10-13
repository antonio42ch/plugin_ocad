//! Motor de geometría del plugin Geometría.
//!
//! Igual que el crate `landsurvey` del plugin de referencia, aquí vive toda la
//! matemática y el parseo, sin depender del host ni del codec CAD (solo `std`).
//! Así se puede probar con `cargo test -p geometria` sin abrir Open CAD Studio.

use std::f64::consts::{PI, TAU};

/// Un punto 2D (x, y).
pub type Punto2 = (f64, f64);

/// Vértices de un polígono regular de `lados` lados inscrito en una
/// circunferencia de `radio`, centrado en `centro`. El primer vértice apunta
/// hacia arriba (+Y). Devuelve `None` si los parámetros no son válidos.
pub fn poligono_regular(lados: usize, radio: f64, centro: Punto2) -> Option<Vec<Punto2>> {
    if lados < 3 || !(radio.is_finite() && radio > 0.0) {
        return None;
    }
    let inicio = PI / 2.0;
    Some(
        (0..lados)
            .map(|i| {
                let a = inicio + TAU * i as f64 / lados as f64;
                (centro.0 + radio * a.cos(), centro.1 + radio * a.sin())
            })
            .collect(),
    )
}

/// Vértices de una estrella de `puntas` puntas: alterna entre el radio exterior
/// y el interior. Devuelve `None` si los parámetros no son válidos.
pub fn estrella(puntas: usize, r_ext: f64, r_int: f64, centro: Punto2) -> Option<Vec<Punto2>> {
    if puntas < 2 || !(r_ext > 0.0 && r_int > 0.0 && r_int < r_ext) {
        return None;
    }
    let n = puntas * 2;
    let inicio = PI / 2.0;
    Some(
        (0..n)
            .map(|i| {
                let r = if i % 2 == 0 { r_ext } else { r_int };
                let a = inicio + TAU * i as f64 / n as f64;
                (centro.0 + r * a.cos(), centro.1 + r * a.sin())
            })
            .collect(),
    )
}

/// Segmentos de una rejilla de `nx` × `ny` celdas de lado `paso` con la esquina
/// inferior izquierda en `origen`.
pub fn rejilla(nx: usize, ny: usize, paso: f64, origen: Punto2) -> Option<Vec<(Punto2, Punto2)>> {
    if nx == 0 || ny == 0 || !(paso > 0.0) {
        return None;
    }
    let (x0, y0) = origen;
    let (ancho, alto) = (nx as f64 * paso, ny as f64 * paso);
    let verticales = (0..=nx).map(|i| {
        let x = x0 + i as f64 * paso;
        ((x, y0), (x, y0 + alto))
    });
    let horizontales = (0..=ny).map(|j| {
        let y = y0 + j as f64 * paso;
        ((x0, y), (x0 + ancho, y))
    });
    Some(verticales.chain(horizontales).collect())
}

/// Área de un polígono cerrado (fórmula del zapatero), siempre positiva.
pub fn area(vertices: &[Punto2]) -> f64 {
    let n = vertices.len();
    let doble: f64 = (0..n)
        .map(|i| {
            let (x1, y1) = vertices[i];
            let (x2, y2) = vertices[(i + 1) % n];
            x1 * y2 - x2 * y1
        })
        .sum();
    doble.abs() / 2.0
}

/// Perímetro de un polígono cerrado.
pub fn perimetro(vertices: &[Punto2]) -> f64 {
    let n = vertices.len();
    (0..n)
        .map(|i| {
            let (x1, y1) = vertices[i];
            let (x2, y2) = vertices[(i + 1) % n];
            (x2 - x1).hypot(y2 - y1)
        })
        .sum()
}

/// Un punto leído de un CSV `nombre,x,y[,z][,descripcion]`.
#[derive(Debug, Clone, PartialEq)]
pub struct PuntoCsv {
    pub nombre: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub descripcion: String,
}

/// Resultado de leer un CSV de puntos.
#[derive(Debug, Default)]
pub struct LecturaCsv {
    pub puntos: Vec<PuntoCsv>,
    /// Números de línea (desde 1) que no se pudieron leer.
    pub omitidas: Vec<usize>,
}

/// Lee un CSV de puntos `nombre,x,y[,z][,descripcion]`. Acepta coma, punto y
/// coma o tabulador como separador. Las líneas vacías, los comentarios (`#`) y
/// una cabecera no numérica se ignoran; las demás líneas inválidas se cuentan.
pub fn leer_csv(texto: &str) -> LecturaCsv {
    let mut out = LecturaCsv::default();
    for (i, linea) in texto.lines().enumerate() {
        let linea = linea.trim();
        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }
        let campos: Vec<&str> = linea.split([',', ';', '\t']).map(str::trim).collect();
        let num = |k: usize| campos.get(k).and_then(|s| s.parse::<f64>().ok()).filter(|v| v.is_finite());
        match (num(1), num(2)) {
            (Some(x), Some(y)) => out.puntos.push(PuntoCsv {
                nombre: campos[0].to_string(),
                x,
                y,
                z: num(3).unwrap_or(0.0),
                descripcion: campos.get(4).copied().unwrap_or("").to_string(),
            }),
            // La primera línea no numérica es la cabecera.
            _ if i == 0 => {}
            _ => out.omitidas.push(i + 1),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn cuadrado_inscrito() {
        let v = poligono_regular(4, 2f64.sqrt(), (0.0, 0.0)).unwrap();
        assert_eq!(v.len(), 4);
        assert!(casi(v[0].0, 0.0) && casi(v[0].1, 2f64.sqrt()));
        assert!(casi(area(&v), 4.0));
        assert!(casi(perimetro(&v), 8.0));
    }

    #[test]
    fn hexagono_con_centro() {
        let v = poligono_regular(6, 1.0, (10.0, 5.0)).unwrap();
        // Área del hexágono regular de radio 1: 3·√3/2.
        assert!(casi(area(&v), 3.0 * 3f64.sqrt() / 2.0));
        assert!(casi(perimetro(&v), 6.0));
    }

    #[test]
    fn parametros_invalidos() {
        assert!(poligono_regular(2, 1.0, (0.0, 0.0)).is_none());
        assert!(poligono_regular(5, -1.0, (0.0, 0.0)).is_none());
        assert!(estrella(5, 1.0, 2.0, (0.0, 0.0)).is_none());
        assert!(rejilla(0, 3, 1.0, (0.0, 0.0)).is_none());
    }

    #[test]
    fn estrella_alterna_radios() {
        let v = estrella(5, 10.0, 4.0, (0.0, 0.0)).unwrap();
        assert_eq!(v.len(), 10);
        assert!(casi(v[0].0.hypot(v[0].1), 10.0));
        assert!(casi(v[1].0.hypot(v[1].1), 4.0));
    }

    #[test]
    fn rejilla_cuenta_lineas() {
        let s = rejilla(3, 2, 5.0, (0.0, 0.0)).unwrap();
        assert_eq!(s.len(), 4 + 3);
        assert_eq!(s[3], ((15.0, 0.0), (15.0, 10.0)));
    }

    #[test]
    fn csv_con_cabecera_y_errores() {
        let l = leer_csv("nombre,x,y,z,desc\nP1,1,2,3,arbol\n\n# nota\nP2;4;5\nmal,a,b\n");
        assert_eq!(l.puntos.len(), 2);
        assert_eq!(l.puntos[0].descripcion, "arbol");
        assert_eq!(l.puntos[1].z, 0.0);
        assert_eq!(l.omitidas, vec![6]);
    }
}
