---
source: 46b4723cec5ef9109b70a2a54e850458f119c7e9
---

# Ejecutar Godot desde el editor { #running-godot-from-the-editor }

Los guardados, las construcciones y el modo en vivo hablan con el addon por el Live Link, así que funcionan con cualquier editor de Godot que tenga el proyecto abierto. TrenchDot solo necesita encontrar el ejecutable de Godot para *Godot > Abrir proyecto en el editor de Godot*, el robot de Godot de la barra de herramientas y *Godot > Ejecutar proyecto* (F5) mientras ningún editor de Godot tenga el proyecto abierto, ya que esos inician Godot por sí mismos.

![El menú Godot con Ejecutar proyecto resaltado, su atajo F5 a la derecha y Abrir proyecto en el editor de Godot debajo](../assets/running-godot/menu-run-project.png)

## Ejecutar proyecto { #run-project }

Mientras un editor de Godot tenga el proyecto abierto, *Ejecutar proyecto* le pide a ese editor que guarde sus escenas abiertas y reproduzca la escena principal, como al pulsar Play en Godot. El juego tiene entonces todo lo que Godot construyó, ediciones en vivo incluidas. La barra de estado debería decir "Godot guardó sus escenas abiertas y ejecuta la escena principal". Si el proyecto no tiene escena principal, lo dice, ponla en Godot en *Project > Project Settings > Application > Run*.

Sin un editor de Godot, *Ejecutar proyecto* inicia el juego con el ejecutable de Godot. Entonces ejecuta las escenas tal como se guardaron por última vez en Godot. Lo mismo pasa con un addon más antiguo, y la barra de estado añade que al juego puede faltarle tu último cambio de mapa. Guárdalos en Godot, o pulsa Play allí.

## Encontrar el ejecutable { #finding-the-executable }

Ve a *Archivo > Preferencias* y mira *Ejecutable de Godot*. Cuando TrenchDot encontró uno, dice "Usando" y la ruta debajo del campo.

![El campo Ejecutable de Godot en Preferencias con la línea "Usando" y la ruta debajo](../assets/running-godot/prefs-godot-using.png)

Cuando dice "No encontrado" en amarillo, haz clic en el botón "…" junto al campo y elige en tu disco el ejecutable de Godot, el que empieza por "Godot_v". Haz lo mismo si eligió un Godot equivocado. Hasta que encuentre uno, Abrir proyecto en el editor de Godot sigue en gris, y también Ejecutar proyecto mientras ningún editor de Godot tenga el proyecto abierto.

![El campo Ejecutable de Godot vacío con el botón … y la línea amarilla No encontrado](../assets/running-godot/prefs-godot-not-found.png)

Sin una ruta válida en Preferencias, el editor toma la primera coincidencia de:

1. La variable de entorno `GODOT`.
2. `godot` o `godot4` en el `PATH`, y luego las builds descargadas como `Godot_v4.7.2-stable_win64.exe`, la más nueva primero. Las builds Mono ganan cuando la carpeta del proyecto tiene un `.csproj`, ya que un proyecto C# las necesita. Las builds de consola se saltan.
3. Carpetas de instalación habituales: WinGet, Scoop, Steam y `Programs/Godot` en Windows, `~/.local/bin` y Steam en Linux, `/Applications/Godot.app` en macOS.

## El botón de Godot { #the-godot-button }

El robot de Godot del extremo derecho de la barra de herramientas muestra la conexión. Pasa el ratón por encima para ver qué haría.

Gris significa que no hay ningún proyecto abierto o que no se encontró Godot, así que hacer clic no hace nada.

![El robot de Godot gris con el ratón encima, su tooltip diciendo que no se encontró Godot](../assets/running-godot/toolbar-godot-not-found.png)

Normal significa que se encontró Godot. Haz clic para abrir el proyecto en el editor de Godot.

![El robot de Godot normal con el ratón encima, su tooltip diciendo Abrir proyecto en el editor de Godot](../assets/running-godot/toolbar-godot-found.png)

Cian significa que un editor de Godot tiene este proyecto abierto. Haz clic para traer ese editor al frente.

![El robot de Godot cian con el ratón encima, su tooltip diciendo que Godot 4.7.2-stable (official) tiene este proyecto abierto, clic para mostrarlo](../assets/running-godot/toolbar-godot-open.png)

Junto a él aparece una ruedecita mientras Godot construye. Si el tooltip dice que el Godot en marcha tiene un addon más antiguo, actualízalo con *Godot > Instalar o actualizar addon…* para tener el modo en vivo.

## Confianza en el proyecto { #project-trust }

Godot ejecuta los scripts de un proyecto en cuanto lo abre, lo reproduce o lo construye, y un proyecto o un paquete de mapas que descargaste puede traer scripts propios. Así que cuando abres un proyecto que TrenchDot aún no conoce y tiene código, TrenchDot pregunta primero.

TrenchDot primero busca código en el proyecto. Mientras lo hace, la barra de estado muestra *Buscando código* y no se inicia nada en Godot sobre el proyecto, así que un proyecto grande tarda un momento antes de preguntar o abrirse.

![El aviso que pregunta "¿Confías en los autores de los archivos de este proyecto?" con la carpeta del proyecto, lo que bloquea el modo restringido, la casilla de la carpeta superior y los botones Confiar en el proyecto y Abrir en modo restringido](../assets/running-godot/trust-prompt.png)

Haz clic en **Confiar en el proyecto** para tus proyectos y los de gente de confianza. Marca antes *Confiar en los autores de todos los archivos de la carpeta superior* para confiar en todos los proyectos de esa carpeta, como donde guardas todos tus juegos. Sin eso confías solo en ese proyecto, y un proyecto desempaquetado dentro, como un paquete de mapas con su propio `project.godot`, no recibe esa confianza.

Haz clic en **Abrir en modo restringido** cuando no estés seguro. Puedes ver y editar los mapas, pero TrenchDot no inicia Godot sobre el proyecto, no construye en Godot ni envía ediciones en vivo. *Modo restringido* aparece entonces en amarillo a la derecha de la barra de estado, haz clic para responder la pregunta otra vez.

![El chip amarillo Modo restringido en la barra de estado, su tooltip diciendo que TrenchDot no ejecuta este proyecto en Godot hasta que confíes en él](../assets/running-godot/restricted-mode-chip.png)

En modo restringido, *Ejecutar proyecto*, el robot de Godot, *Construir en Godot* y activar el modo en vivo preguntan "Esto ejecuta sus scripts. ¿Confiar y continuar?", y siguen adelante en cuanto haces clic en **Confiar en el proyecto**.

Abrir un mapa que pertenece a otro proyecto en el que no confías mantiene el proyecto abierto, y el aviso añade **Mantener el proyecto actual** a sus botones. Un `project.godot` dentro de la carpeta del proyecto abierto se ignora, como lo ignora Godot. Cuando un mapa de fuera del proyecto abierto ejecuta GDScript propio, la línea de estado lo dice y el panel Problemas muestra dónde.

Los proyectos de *Proyectos recientes* de antes de que TrenchDot preguntara por la confianza cuentan como de confianza, y también un proyecto con el que inicias el editor mediante `--project`, durante esa sesión.

> [!NOTE]
> El modo restringido cubre solo lo que inicia TrenchDot. Abrir tú mismo el proyecto en Godot ejecuta sus scripts como siempre.

### Proyectos sin código { #projects-without-code }

Un proyecto que no tiene código se abre sin la pregunta, ya que Godot no tiene nada que ejecutar en él. Eso es una carpeta vacía, un proyecto recién creado, o uno con solo el addon de TrenchDot y el paquete de entidades de gameplay que instaló. Aquí código significa scripts, proyectos C#, bibliotecas nativas, escenas y recursos con scripts, autoloads, plugins del editor y mapas cuyo `logic_script` contiene GDScript. El addon y el paquete solo cuentan como propios de TrenchDot mientras cada archivo en ellos sea el que instala esta versión, así que una copia editada o más antigua cuenta como código, y también un proyecto demasiado grande para revisarlo rápido.

TrenchDot vuelve a mirar cada vez que abre el proyecto y no recuerda nada, así que pregunta cuando el proyecto gana scripts. Para mantener tu propio proyecto como de confianza, guarda un mapa en él con *Archivo > Guardar* mientras no tenga código. Desde entonces TrenchDot confía en él, también después de que añadas scripts en Godot. Guardar un mapa en un proyecto en modo restringido no da confianza a nada.

### Proyectos de confianza { #trusted-projects }

Abre *Archivo > Preferencias…* y mira *Proyectos de confianza* abajo del todo para ver cada proyecto y carpeta en los que confías. Haz clic en **Quitar** junto a uno para retirar la confianza. Cuando es el proyecto abierto, pasa a modo restringido al instante y el chip amarillo aparece en la barra de estado, salvo que el proyecto no tenga código.

![La parte Proyectos de confianza de Preferencias con dos proyectos y una carpeta con todos sus proyectos, cada uno con un botón Quitar, el tooltip de uno diciendo que TrenchDot vuelve a preguntar antes de que Godot ejecute aquí los scripts](../assets/running-godot/trusted-projects.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

Un agente nunca se encuentra con el aviso en su camino. `open_project` y `get_state` informan `restricted`, y las herramientas que harían que Godot ejecute o construya el proyecto fallan con un error hasta que confíes en él con el chip. `map_file` `open` mantiene el proyecto abierto para un mapa de uno sin confianza y devuelve ese proyecto como `other_project`.

`trust` en los dos resultados dice por qué un proyecto no está en modo restringido: `stored`, `session` para `--project`, o `no_code`.

Los guardados que hace un agente, con `map_file`, `run_action` o dentro de `run_script`, nunca guardan la confianza, así que un proyecto sin código en el que trabaja un agente se vuelve a preguntar cuando tenga scripts.

</div>
