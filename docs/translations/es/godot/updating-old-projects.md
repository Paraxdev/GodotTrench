---
source: 4c71c99f5fabec89f69f0c99554b4f3a60fb4148
---

# Actualizar un proyecto de GodotTrench { #updating-a-godottrench-project }

TrenchDot se llamaba antes GodotTrench, y las builds beta se llamaron brevemente LevelBench. Si montaste un proyecto o guardaste mapas entonces, siguen funcionando. Solo tienes que actualizar el addon una vez.

## Pasos { #steps }

1. Inicia el editor nuevo. En el primer arranque copia tus preferencias, proyectos de confianza y archivos recientes de la carpeta de ajustes antigua. La carpeta antigua se queda donde está, así que el editor antiguo también sigue funcionando.
2. Abre tu proyecto de Godot. Se abre la ventana de configuración y dice que el addon del proyecto es más antiguo que el editor, incluso cuando los números de versión coinciden. Cierra Godot primero, y pulsa **Actualizar el addon a** seguido de la versión.
3. Abre el proyecto en Godot una vez. El addon mueve sus ajustes de proyecto de `godottrench/` a `trenchdot/` y escribe `trenchdot_game.json` en lugar de `godottrench_game.json`.

## Qué se conserva { #what-carries-over }

| Tenías | Qué pasa |
| --- | --- |
| Mapas `.gtm` | Se abren tal cual. La línea de estado dice "room.gtm actualizado desde el formato GodotTrench, guarda para conservarlo". Guardar mantiene el nombre `.gtm`, así que tus escenas siguen encontrando el mapa. Los mapas nuevos se guardan como `.tdm`. |
| Variables de entorno `GODOTTRENCH_` | Siguen funcionando. Cuando están puestos los dos nombres, gana el de `TRENCHDOT_`. |
| El paquete de gameplay en `res://godottrench/entities` | Sigue funcionando, y tus propios scripts que extienden sus clases siguen compilando. Si instalas también el paquete nuevo, el nuevo gana donde los dos definen una entidad. Deja la carpeta antigua en su sitio mientras tus escenas la usen. |
| Paquetes de naturaleza y modelos en `res://godottrench/` | Los mapas que los colocan siguen funcionando. Los presets de scatter buscan en `res://trenchdot/nature`, así que añade de nuevo el paquete de naturaleza para pintar scatter nuevo. |
| Escenas construidas por el addon antiguo | Se cargan, porque Godot encuentra los scripts del addon movido por su uid. Construir el mapa otra vez las refresca. |

<div class="td-mcp" markdown>

## Clientes MCP { #mcp-clients }

El servidor MCP ahora se llama `trenchdot`. Cámbiale el nombre en la configuración de tu cliente. La variable del token es `TRENCHDOT_MCP_TOKEN`, y la antigua `GODOTTRENCH_MCP_TOKEN` sigue funcionando.

</div>
