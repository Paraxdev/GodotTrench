---
source: 77812fdfe3757a8be2d8e1be3d7fbd73a1ff678a
---

# Revisar cambios { #reviewing-changes }

Guarda tus mapas en git y podrás ver qué cambió entre dos versiones, igual que revisas código. Para los cambios dentro de la sesión actual, el [panel Historial](organizing.md#history) lista cada edición y deshace hasta cualquiera de ellas.

## Mapas en git { #maps-in-git }

Los archivos `.tdm` son binarios, pero git aún puede mostrarte diffs legibles. Le pide al editor que imprima cada versión como JSON y compara esos textos. Lo configuras una sola vez:

1. Añade esta línea a `.gitattributes` en tu proyecto:

    ```
    *.tdm binary diff=tdm
    ```

2. Dile a git cómo imprimir un mapa, una vez por clon:

    ```sh
    git config diff.tdm.textconv "trenchdot --dump"
    ```

3. (Opcional) Deja las copias de seguridad y los autoguardados fuera del repositorio con estas líneas en `.gitignore`:

    ```
    *.tdm.bak
    *.tdm.autosave
    ```

`git diff` y `git log -p` ahora deberían mostrar los cambios de un mapa como JSON, en lugar de limitarse a decir que los archivos binarios difieren.

> [!WARNING]
> `trenchdot` tiene que estar en tu `PATH`, o `git diff` falla con los mapas.

Para obtener ese JSON como archivo, `trenchdot --to-json map.tdm map.json` lo escribe, y `trenchdot --to-tdm map.json
map.tdm` lo convierte de nuevo en un mapa binario. Mira [Línea de comandos](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Las herramientas de revisión de MCP te muestran qué le hizo un agente de IA a tu mapa.

### Ediciones hechas por un agente de IA { #edits-made-by-an-ai-agent }

Cuando un agente edita tu mapa por [MCP](../mcp.md), cada llamada a una herramienta es un paso de deshacer llamado `MCP: ...` en el [panel Historial](organizing.md#history). Un `run_script` entero también es un solo paso, como `MCP: Add crates, 6 steps`. Deshaz ese paso para retirar el trabajo del agente.

![El panel Historial con MCP: Add crates, 6 steps arriba, sobre ediciones hechas a mano como Rename, Group y Hollow](../assets/reviewing/history-mcp-script.png)

| Para ver | Pídele al agente |
| --- | --- |
| Qué cambió desde el último guardado | `changes_since`, que lista los nodos añadidos, quitados, movidos o editados de otra forma |
| Qué cambió su última llamada o script | `changes_since` con `undo_steps: 1` |
| Qué cambió desde un commit de git | `changes_since` con `file` apuntando a esa versión, escrita con `git show HEAD:maps/level.tdm > old.tdm` |

> [!TIP]
> Pídele al agente que ponga un trabajo de varios pasos en un solo `run_script` con un `label`, para que todo el trabajo se deshaga en un paso con un nombre que reconozcas.

</div>
