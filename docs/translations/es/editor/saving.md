---
source: 320dd2516530d568c7e2acda31113f93ead3f05b
---

# Guardado y recuperación { #saving-and-recovery }

Pulsa Ctrl+S para guardar, o Ctrl+Shift+S para *Guardar como…*. Los diálogos de archivo empiezan en la carpeta del mapa actual, o en el proyecto de Godot si el mapa nunca se guardó, ya que el addon carga los mapas por su ruta `res://`. *Archivo > Mapas del proyecto* lista todos los mapas del proyecto si solo quieres abrir uno rápido.

## Copias de seguridad y autoguardado { #backups-and-autosave }

El editor guarda dos tipos de copias de repuesto junto a tu mapa:

| Archivo | Cuándo se escribe |
| --- | --- |
| `<map>.tdm.bak` | En cada guardado, una copia de la versión anterior. Renómbrala a `.tdm` para volver un guardado atrás |
| `<map>.tdm.autosave` | Cada tres minutos, para cada pestaña con cambios sin guardar |

Para cambiar la frecuencia del autoguardado, ve a *Archivo > Preferencias* y ajusta *Autoguardado (minutos, 0 = apagado)*.

![El campo Autoguardado (minutos, 0 = apagado) en Preferencias con el valor 3](../assets/saving/prefs-autosave.png)

Los autoguardados usan una extensión que Godot no importa, así que un mapa a medio hacer nunca aparece en tu juego. Guardar el mapa borra su autoguardado.

## Recuperar un mapa { #recovering-a-map }

Si el editor se cerró antes de que guardaras, abre el autoguardado:

1. Elige *Archivo > Abrir mapa…*.
2. Escoge el tipo de archivo **Autoguardado (recuperar un mapa)** y abre el archivo `.tdm.autosave`. La barra de estado debería decir que recuperó el mapa de su autoguardado.
3. Guarda. Así se vuelve a escribir el `.tdm` real.

Los mapas que nunca guardaste se autoguardan en un archivo temporal. Los encuentras en *Archivo > Abrir reciente*, debajo de *Mapas sin título guardados automáticamente*.

![Archivo > Abrir reciente con Sin mapas recientes arriba y debajo una lista de mapas sin título guardados automáticamente](../assets/saving/open-recent-autosaves.png)

## Archivos dañados { #damaged-files }

Un archivo de mapa dañado en el disco todavía se abre. El editor lee cada parte intacta, mete los objetos cuya capa, grupo o entidad se perdió en una capa llamada *Recovered*, y dice en la barra de estado qué no se pudo leer. Al guardar se conserva el original dañado como `<map>.tdm.bak`, así que aún puedes restaurarlo desde el control de versiones.
