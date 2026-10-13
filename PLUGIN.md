# Geometría (`opencad.geometria`)

Plugin externo de ejemplo para [Open CAD Studio](https://github.com/HakanSeven12/OpenCADStudio),
construido con la misma arquitectura que
[Land Survey](https://github.com/KevinGriffin-new/opencad-landsurvey-plugin).
La matemática vive en el crate `geometria` (`crates/geometria/`), sin dependencias del host.

## Comandos

| Comando | Origen | Descripción |
|---------|--------|-------------|
| `GM_HOLA` | ribbon / línea de comandos | Muestra los comandos disponibles. |
| `GM_POLIGONO [<lados> <radio> [cx cy]]` | ribbon / línea de comandos | Polígono regular cerrado en `GM-FIGURAS`, etiquetado en `GM-ETIQUETAS`. **Sin argumentos es interactivo:** clic en el centro, el host previsualiza el polígono al mover el cursor, clic para fijar el radio. |
| `GM_LADOS [<n>]` | línea de comandos | Lados del polígono interactivo (por defecto 6). Sin argumento, los muestra. |
| `GM_ESTRELLA <puntas> <radio_ext> <radio_int> [cx cy]` | ribbon (muestra el uso) / línea de comandos | Estrella cerrada en `GM-FIGURAS`. |
| `GM_REJILLA <nx> <ny> <paso> [x0 y0]` | ribbon (muestra el uso) / línea de comandos | Rejilla de líneas en `GM-REJILLA`, añadidas en un solo lote (`add_entities`). |
| `GM_PUNTOS <ruta>` | ribbon (selector de archivos) | Importa un CSV `nombre,x,y[,z][,descripcion]` (coma, `;` o tabulador) como entidades `Point` en `GM-PUNTOS` con etiqueta. Ver `ejemplos/puntos.csv`. |
| `GM_LISTAR` | ribbon / línea de comandos | Relee del dibujo las figuras (área y perímetro de su geometría *actual*) y los puntos del plugin. |

## XDATA

| Aplicación | En | Valores |
|------------|----|---------|
| `GEOMETRIA_FIGURA` | cada polígono/estrella (LwPolyline) | `[String(nombre), String(tipo)]` — tipo `POLIGONO` / `ESTRELLA` |
| `GEOMETRIA_PUNTO` | cada `Point` importado | `[String(nombre), String(descripcion)]` |
