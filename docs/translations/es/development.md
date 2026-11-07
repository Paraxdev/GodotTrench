---
source: d0d174df9061792086d11223ac4029e9d23f9d56
---

# Desarrollo { #development }

Esta página es para ti si quieres compilar el editor desde el código fuente, cambiar el addon o escribir documentación.

## Compilar { #building }

Clona con los submódulos, el addon de Godot en `godot/addons/func_godot` es uno:

```sh
git clone --recursive https://github.com/Paraxdev/TrenchDot.git
```

¿Ya clonaste sin ellos? Ejecuta `git submodule update --init`. El editor compila sin el addon, pero las pruebas y el proyecto de Godot lo necesitan.

```sh
cargo run -p td_editor --release            # start the editor
cargo run -p td_editor --release -- map.tdm # open a map
```

Necesitas una GPU con Vulkan, DX12 o Metal, y en Linux una sesión de X11 o Wayland.

Rust 1.95 es el mínimo, egui 0.36 lo exige.
[rust-toolchain.toml](https://github.com/Paraxdev/TrenchDot/blob/main/rust-toolchain.toml) fija la 1.98.1, la versión que usa CI, así que rustup la instala en tu primera compilación.

## Comprobaciones { #checks }

Ejecuta `./tools/fmt --install` una vez por clon. Activa un hook de pre-commit que da formato al Rust preparado para el commit.

```sh
./tools/fmt --check
python tools/gen_licenses.py --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p td_editor --test e2e -- --ignored --test-threads=1
godot --headless --path godot --import                            # twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd
godot --headless --path godot --script res://tests/build_showcase.gd
```

CI ejecuta todo esto, así que ejecútalo antes de hacer push.

Las pruebas e2e lanzan el editor real y lo controlan con entrada simulada. Eso necesita una GPU y una sesión de escritorio, así que `cargo test` las omite salvo que pases `--ignored`. CI las ejecuta en Linux con Vulkan por software. Para probar el Live Link sin ventana, mira [Protocolo del Live Link](godot/live-link.md#testing-without-a-window).

> [!NOTE]
> `godot --headless --script` no renderiza nada (las capturas y las comprobaciones de render salen negras) y no ejecuta los autoloads del proyecto, solo el propio script. Escribe las pruebas sobre datos y nodos, no sobre píxeles ni sobre el estado de un autoload.

Para medir el tiempo de construcción de mapas, ejecuta `godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`.
No uses `build_showcase.gd` para eso, sobrescribe las escenas de muestra incluidas en el repositorio.

## Archivos de mapa en git { #map-files-in-git }

Los mapas `.tdm` son binarios, mira [el formato de mapa .tdm](format/README.md). El `.gitattributes` del repositorio ya los marca. Para tener diffs legibles, solo necesitas la línea de `git config` de [Revisar cambios](editor/reviewing.md#maps-in-git), una vez por clon.

## Contenido generado { #generated-content }

Algunos archivos los hace un script, no una persona. Cuando cambies lo que lee un script, ejecútalo de nuevo.

| Comando | Qué regenera |
| --- | --- |
| `cargo run -p td_formats --example demo_maps` | Los mapas de prueba de `godot/tests/maps` y los pequeños mapas de demo de `godot/demo/maps` |
| `python tools/fetch_demo_textures.py` | Las texturas fotográficas CC0 de la demo, mira [tamaño de textura](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | Los props CC0 de Poly Haven de `godot/models/polyhaven`, con los mapas alfa del cristal y las llamas restaurados. `--fix-alpha` solo los restaura |
| `cargo run -p td_samples --bin showcase --release` | Los modelos de Blockbench de la demo |
| `cargo run -p td_editor --example export_addon_fgd` | Los archivos FGD básicos del addon y la plantilla del paquete de entidades de gameplay a partir de `builtin_entities.json` y `gameplay_entities.json`, escritos en el submódulo del addon, y luego la copia instalada del paquete en el proyecto de demo |
| `python tools/gen_licenses.py` | La tabla de crates de Rust de [LICENSES.md](https://github.com/Paraxdev/TrenchDot/blob/main/LICENSES.md) |

## El addon de Godot { #the-godot-addon }

El addon tiene su propio repositorio, [trenchdot_func](https://github.com/Paraxdev/trenchdot_func), clonado en `godot/addons/func_godot`. Sus pruebas viven aquí, en `godot/tests`, porque necesitan el proyecto de demo. Para cambiar el addon:

1. Haz commit del cambio dentro de `godot/addons/func_godot` y haz push a trenchdot_func.
2. Prepara aquí el nuevo commit del submódulo con `git add godot/addons/func_godot` y haz commit junto con los cambios de TrenchDot que lo necesiten.

Haz push del addon primero. Un commit de TrenchDot que apunta a un commit del addon que GitHub no tiene falla al hacer checkout en CI.

El `version` de `plugin.cfg` coincide con la versión del workspace de [Cargo.toml](https://github.com/Paraxdev/TrenchDot/blob/main/Cargo.toml). Sube las dos a la vez, el editor avisa cuando el addon de un proyecto no coincide con ella.

## Traducciones { #translations }

La interfaz del editor viene en inglés, alemán, francés, español, portugués, ruso, chino simplificado y japonés, mira [Idioma](editor/interface.md#language). El inglés es la fuente, y los otros siete viven en
[crates/td_editor/locales](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales), un archivo JSON cada uno, incrustado al compilar el editor. El README de esa carpeta tiene el formato, las formas del plural y un glosario.

En el código, el texto en inglés es la clave. Envuelve un literal en `en!("...")` donde se guarda y en `tr(...)`, `trf!("... {n} ...", n = count)` o `tr_n!(count, "{n} brush", "{n} brushes")` donde se dibuja, y añade la misma clave a todos los archivos. Un texto sin entrada se muestra en inglés. El código de `crates/td_editor/src/mcp`, las líneas de registro y el contenido de los mapas nunca pasan por ellos, las respuestas y los errores de MCP siguen en inglés. Un `Msg` guarda el texto en inglés junto a la traducción, así la barra de estado puede mostrar uno mientras MCP lee el otro.

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor --lib i18n
```

Esa ejecución falla mientras a un archivo le falte una clave que usa el código o conserve una que ya no. Con `TD_L10N_TODO` apuntando a una carpeta existente también escribe un archivo por idioma con las claves que faltan. Otra prueba recorre el código fuente en busca de texto de interfaz que se salte la búsqueda, así que un literal en bruto pasado a una etiqueta la hace fallar. Las traducciones las escribe una IA y nadie las ha revisado, una corrección de un hablante nativo solo toca el archivo de ese idioma. El editor iniciado con `--default-prefs`, como hacen las pruebas e2e, está siempre en inglés.

El manual también tiene traducciones, mira [Traducir el manual](#translating-the-manual). Sus páginas nombran los elementos de menú y los botones como los muestra el editor, así que después de cambiar un valor en un catálogo ejecuta `python3 tools/translations.py check`. Lista las páginas que todavía usan la redacción antigua.

## Documentación { #documentation }

La documentación son las páginas Markdown de `docs/`, y [Zensical](https://zensical.org) las convierte en el sitio. `zensical.toml` en la raíz del repositorio lo configura, y `docs/SUMMARY.md` es el índice. Una página que no está ahí se construye igualmente, solo que sin lugar en la navegación. `docs/requirements.txt` fija Zensical, `docs/stylesheets/td.css` guarda los colores, `docs/javascripts/td.js` el interruptor de MCP y las tarjetas del glosario, y `docs/overrides` reemplaza algunas plantillas del tema.

Ejecuta esto desde la raíz del repositorio:

```sh
python3 -m venv .venv                  # once
. .venv/bin/activate                   # .venv\Scripts\activate on Windows
pip install -r docs/requirements.txt   # once, and again when the pin changes
zensical serve                         # live preview of the English site on http://localhost:8000/TrenchDot/
python3 tools/translations.py check    # the translated pages against their English source
python3 tools/build_site.py            # the build CI runs, English at the root of _site and a folder per language
python3 tools/test_docs.py             # tests of the three tools and of the built site
python3 tools/check_site.py            # the link check CI runs on _site
```

`build_site.py` ejecuta `zensical build --clean --strict` una vez para el inglés y otra para cada idioma, así que falla ante un enlace a una página o un encabezado que no existe en ninguno. Una entrada mal escrita de `SUMMARY.md`, una imagen que falta o una página fuera de la navegación se le escapan, `tools/check_site.py` las detecta en todos los idiomas. Para construir solo algunos idiomas, nómbralos: `python3 tools/build_site.py de ja`.

El id de un encabezado es su texto en minúsculas, los espacios pasan a guiones y se descarta el resto de la puntuación, así que `## path_corner` es `#path_corner`. Los enlaces de ayuda del editor en
[help.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/help.rs) apuntan a estos ids, y una prueba los comprueba, así que cambia el nombre de un encabezado solo junto con sus enlaces.

Los avisos son alertas al estilo de GitHub: `> [!NOTE]`, `> [!TIP]` o `> [!WARNING]` en una línea propia, con el texto en las líneas citadas de debajo. Así se ven también en GitHub. Todo lo que pertenece a un paso numerado, un segundo párrafo, una imagen o un bloque de código, se sangra 4 espacios, o se sale de la lista.

Las imágenes de una página van en `docs/assets/<page>/`.

Si alguien nuevo no conocería un término, dale una entrada en `GLOSSARY.md`: un encabezado `## term` con una definición de un párrafo. Su primer uso en cada página enlaza entonces con ella y muestra la definición al pasar el cursor. El código, los enlaces y los encabezados se dejan como están.

`.github/workflows/pages.yml` construye el sitio para los pull requests que tocan `docs/`, `zensical.toml`, las herramientas de la documentación o los catálogos del editor, y lo publica en GitHub Pages cuando llegan a `main`.

### Traducir el manual { #translating-the-manual }

El sitio también se construye en alemán, francés, español, portugués, ruso, chino simplificado y japonés, los idiomas del editor. El inglés es la fuente, y los demás idiomas se llaman como los códigos del editor: `de`, `fr`, `es`, `pt`, `ru`, `zh-Hans` y `ja`. El sitio de cada idioma vive en `/TrenchDot/<code>/`, y el editor abre ahí el manual cuando funciona en ese idioma. Las traducciones las escribe una IA y ningún hablante nativo las ha revisado, así que cada sitio traducido lo dice en un aviso que el lector puede cerrar. Una corrección de alguien que domine el idioma es bienvenida.

**Dónde está cada cosa.** Una página traducida está en `docs/translations/<code>/` con el nombre de la inglesa, `docs/translations/de/editor/brushes.md` para `docs/editor/brushes.md`. Junto a las páginas, `site.toml` guarda los textos que no son páginas: el idioma del tema, la descripción del sitio, los nombres del botón del esquema de color y del interruptor de MCP, el aviso, la nota para las páginas sin traducir, los títulos de las alertas y una tabla `[nav]`. La tabla `[nav]` asigna a cada título inglés de `SUMMARY.md` su traducción, un título que le falta se muestra en inglés. La lista de idiomas es `[[project.extra.alternate]]` en `zensical.toml`.

**La línea source.** Una página traducida empieza con un front matter que nombra la página inglesa que sigue:

```yaml
---
source: 0123456789abcdef0123456789abcdef01234567
---
```

`source` es lo que imprime `git hash-object docs/<page>` para la página inglesa. Mientras coincide, la traducción está al día y la compilación la usa. Cuando la página inglesa cambia ya no coincide, la traducción está desactualizada, y la compilación muestra en su lugar la página inglesa con una breve nota tras el título. Así, editar una página inglesa nunca rompe CI y no exige editar la traducción, la página solo pasa a estar en inglés en los demás sitios hasta que alguien la actualiza. Una página que falta se resuelve igual, así que toda página existe en todos los idiomas y el botón de idioma siempre aterriza en la misma página.

**Comandos.** Todos se ejecutan desde la raíz del repositorio.

| Comando | Qué hace |
| --- | --- |
| `python3 tools/translations.py status` | Lista las páginas desactualizadas, ausentes y huérfanas y los títulos de navegación sin entrada. Nunca falla. Añade `--lang de` para un idioma y `--diff` para ver qué cambió en inglés desde que se tradujo cada página. |
| `python3 tools/translations.py prep --lang de editor/brushes.md` | Imprime lo que necesita un traductor para una página: el front matter, los ids de los encabezados, las etiquetas con su traducción, las palabras del glosario y el texto en inglés. |
| `python3 tools/translations.py check [--lang de] [page ...]` | Falla ante una traducción al día que rompe una regla de las de abajo. Las páginas que nombras se comprueban sea cual sea su estado, así puedes comprobar una antes de que esté al día. |
| `python3 tools/translations.py stamp --lang de editor/brushes.md` | Pone `source` al valor de la página inglesa tal como está ahora y no toca nada más. |

**Traducir o actualizar una página.** Cuatro pasos:

1. Ejecuta `prep` para la página. Si está desactualizada, también muestra el diff del inglés desde el antiguo `source`.
2. Escribe la página en `docs/translations/<code>/`, con el front matter que imprimió `prep` primero. Mantén la estructura inglesa línea por línea: mismos encabezados, listas, imágenes, enlaces, alertas y bloques `td-mcp`. Traduce el texto, el texto alternativo de las imágenes y el texto de los enlaces, nada más, con la voz del idioma, mira más abajo.
3. Ejecuta `check` para la página y corrige lo que informe hasta que quede limpia.
4. Ejecuta `python3 tools/build_site.py`, `python3 tools/check_site.py` y `python3 tools/translations.py check`, y haz commit juntas de las páginas de un idioma.

Usa `stamp` solo cuando el cambio en inglés no necesita nada en la traducción, como una errata, un destino de enlace cambiado o una reformulación que dice lo mismo. No mira la página, así que lee antes el diff del inglés. Un cambio en lo que dice la página necesita que se edite la traducción, y el nuevo `source` llega con ella.

**Voz.** Todos los idiomas mantienen la voz cercana de [getting-started.md](getting-started.md), en la forma que usa ahí la documentación de software:

| Idioma | Tratamiento |
| --- | --- |
| de | du, en minúscula |
| fr | vous |
| es | tú, en español neutro, ustedes para el plural como dice el README del catálogo |
| pt | você, portugués de Brasil |
| ru | вы, en minúscula |
| zh-Hans | 你 |
| ja | forma です/ます, sin pronombre |

**Qué exige `check`.** Las reglas valen para toda traducción al día, y `check` se ejecuta antes de la compilación en CI.

* Los ids de los encabezados siguen siendo los ingleses. Cada encabezado, también el título, lleva el id de su encabezado inglés, `## Größe ändern { #reshape }`, en el mismo orden y con los mismos sufijos de repetición (`{ #inputs_1 }`). Zensical descarta las letras fuera de ASCII al crear un id, y los enlaces F1 del editor, los enlaces entre páginas, el glosario y el botón de idioma dependen de los ingleses.
* El código sigue en inglés y byte por byte. Los bloques de código se copian con sus comentarios, y todo fragmento de código en línea de la página inglesa aparece en la traducción. Los classnames de entidades, las claves, las Entradas y Salidas, los nombres de herramientas MCP, el JSON, los nombres de archivo, los nombres de teclas (`Ctrl`, `F1`) y los nombres de producto (TrenchDot, Godot, MCP, Live Link) tampoco se traducen.
* Los enlaces y las imágenes conservan sus destinos, solo cambian el texto del enlace y el texto alternativo. Las capturas son las inglesas, una página traducida apunta a los mismos archivos, así que el sitio publicado guarda una copia de cada una.
* Las etiquetas son las del editor. Un fragmento en negrita, o en cursiva que contenga ` > `, cuyas partes sean todas claves del catálogo `crates/td_editor/locales/<code>.json` tiene que aparecer en la página traducida tal como lo escribe el catálogo, con las partes unidas por ` > `. Escribe la etiqueta como la muestra el editor, sin declinar, y construye la frase a su alrededor. Una etiqueta que no está en el catálogo, como un botón de Godot o del addon, sigue en inglés.
* Las alertas, los bloques `td-mcp` y su encabezado `## MCP` se quedan donde están. Escribe el marcador de la alerta solo en su línea, `> [!NOTE]`, la compilación pone detrás el título traducido. Los bloques de GDScript conservan su etiqueta `gdscript`.
* Nada de raya, semirraya, guion con espacios ni emojis, las reglas de las páginas inglesas valen en todos los idiomas.
* En chino y japonés cada párrafo, elemento de lista y celda de tabla es una sola línea de código fuente. Un salto de línea entre dos caracteres CJK se muestra como un espacio en el navegador, así que `check` lo marca. La puntuación de ancho completo está bien, pero los caracteres de raya doble U+2014 y U+2015 cuentan como rayas y no se permiten.

**El glosario.** `GLOSSARY.md` se traduce como cualquier página, y cada término conserva su id inglés, `## Aktivator { #activator }`. Un idioma que declina un término lista las demás formas en `data-forms`, `{ #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }`, para que la tarjeta emergente también las encuentre. Un término que es un nombre de interfaz usa la palabra del catálogo, y `check` las compara. Una página traducida usa un término en la forma propia del glosario donde aparece por primera vez, eso es lo que la enlaza con la tarjeta. Cuando añadas un término al glosario inglés, añádelo a los traducidos si puedes, hasta entonces el glosario de cada idioma está en inglés.

## CI { #ci }

CI omite los push a `main` que solo cambian documentación o Markdown. Cualquier otro push verde a `main` reemplaza la [beta continua](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) con descargas nuevas del editor, el addon, la demo y el contenido.

El editor descarga el addon y el contenido solo de las releases de GitHub. itch.io es una tienda que recibe los mismos archivos por sus propios canales, `addon`, `nature-pack`, `demo-content` y así sucesivamente. La beta continua es la única compilación, así que llena esos canales simples.

| Zip de la release | Se instala con |
| --- | --- |
| `func_godot-trenchdot-addon.zip` | *Godot > Instalar o actualizar addon…* |
| `trenchdot-nature-pack.zip` | *Godot > Añadir contenido al proyecto…* |
| `trenchdot-demo-content.zip` | *Godot > Añadir contenido al proyecto…* |

Las carpetas que contiene cada zip están listadas en
[content.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/content.rs) y
[addon_install.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/addon_install.rs), y las pruebas comprueban contra ellas los comandos de zip de `ci.yml`.

El editor encuentra los zips a través de la API de GitHub. CI lo compila con `TRENCHDOT_CHANNEL=beta`, así que busca primero en la beta continua. Una compilación local prueba primero una release con el nombre de su propia versión, y luego recurre a la beta. Para probar un zip antes de que se publique, apunta el editor a otro servidor con la variable de entorno `TRENCHDOT_RELEASES_API`.

CI también marca cada beta, en `TRENCHDOT_BUILD` para el editor y en el `plugin.cfg` del addon comprimido, como `0.1.0-beta.202611041532.d194b57`: el número de release, la hora UTC del commit y el commit. Así es como el editor distingue un addon de una beta más antigua y ofrece la actualización. Una compilación local conserva la versión simple.

<div class="td-mcp" markdown>

## MCP { #mcp }

¿Contribuyes con un agente de IA? [AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md) lista las reglas y las comprobaciones que tiene que pasar un pull request.

Las pruebas e2e controlan el editor por el [servidor MCP](mcp.md), y `example_mcp_scripts_replay` reproduce todos los scripts de `examples/mcp`.

Para controlar desde la consola un editor tuyo:

```sh
trenchdot --mcp-http=7850 --default-prefs &
TRENCHDOT_MCP_URL=http://127.0.0.1:7850/mcp python3 tools/mcp_client.py call get_state
```

Define `TRENCHDOT_MCP_TOKEN` con el mismo valor en ambos para probarlo con un [token](mcp.md#the-http-token).

### MCP en la documentación { #mcp-in-the-docs }

El interruptor **MCP** de la cabecera, apagado por defecto, oculta todo lo relativo a MCP para que las páginas sigan siendo legibles para quien nunca lo toca. En una página, pon todo el texto de MCP en una sola sección `## MCP` al final de todo, dentro de un bloque `td-mcp`:

```html
<div class="td-mcp" markdown>

## MCP

What only MCP users need.

</div>
```

Mantén las líneas en blanco alrededor del contenido, GitHub solo lee el Markdown de dentro con ellas. Las páginas que tratan solo de MCP van en la parte **MCP** de `SUMMARY.md`, que el interruptor también oculta.

</div>
