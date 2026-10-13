# opencad-geometria-plugin

Plugin de ejemplo para **Open CAD Studio**, basado en la estructura de
[opencad-landsurvey-plugin](https://github.com/KevinGriffin-new/opencad-landsurvey-plugin).
Añade una pestaña **Geometría** al ribbon con comandos `GM_` (ver [PLUGIN.md](PLUGIN.md)).

## Estructura

```
Cargo.toml            cdylib; ocs_plugin_api + codec fijados a la release v2026.41 del host
rust-toolchain.toml   mismo rustc que el host (1.99.0)
plugin.toml           metadatos que lee el host (refleja el MANIFEST de lib.rs)
src/lib.rs            PluginManifest + BuiltinPlugin + export_plugin!
src/ribbon.rs         pestaña del ribbon (CadModule)
src/dispatch.rs       enrutado de comandos GM_ → entidades + XDATA
src/interactivo.rs    GM_POLIGONO con clics y previsualización (InteractiveCommand)
src/state.rs          estado del proceso del plugin
crates/geometria/     motor sin host: polígonos, estrellas, rejillas, CSV (+ tests)
ejemplos/puntos.csv   datos de prueba para GM_PUNTOS
```

## Compilar y probar

```sh
cargo test -p geometria     # tests del motor, sin abrir Open CAD Studio
cargo build --release       # → target/release/libopencad_geometria_plugin.so (.dll / .dylib)
```

## Instalar localmente

```sh
DEST="${XDG_CONFIG_HOME:-$HOME/.config}/OpenCADStudio/plugins/opencad.geometria"
mkdir -p "$DEST"
cp target/release/libopencad_geometria_plugin.so plugin.toml "$DEST/"
```

(En Windows: `%APPDATA%\OpenCADStudio\plugins\opencad.geometria\`, con el `.dll`.)
Reinicia Open CAD Studio: aparece la pestaña **Geometría**.

## Compatibilidad

El plugin debe compilarse con el **mismo toolchain y la misma `ocs_plugin_api`**
que el host. Está fijado a **OpenCADStudio v2026.41 (API 7)**. Para otra versión
del host, cambia el `rev` de `ocs_plugin_api` al commit de ese tag, copia el
`rev` del `codec` **literalmente** del `Cargo.toml` de `crates/ocs_plugin_api`
en ese tag, y actualiza `api_version` en `plugin.toml` y `rust-toolchain.toml`.
