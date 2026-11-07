---
source: 3cc68efb1c3e922c9ae3811f442a5d5e642414b6
---

# Trim { #trim }

*Brush > Añadir trim…* coloca tiras de brush a lo largo de los bordes de las caras, sobresaliendo de ellas. En la parte baja de las paredes son zócalos, en la parte alta molduras de corona, alrededor de un suelo un borde, alrededor de una piscina su remate. Aquí tienes una sala lisa antes y después de un solo clic:

![Una sala enlucida con un vano de puerta y sus paredes seleccionadas, todavía sin trim](../assets/trim/room-before.png)

![La misma sala con zócalos oscuros a lo largo de la parte baja de las paredes, hasta el vano de la puerta](../assets/trim/room-after.png)

1. (Opcional) Elige el material del trim

    Pulsa Escape para que no haya nada seleccionado y haz clic en el material del panel Materiales. Pasa a ser el material actual sin cambiar el mapa.

2. Selecciona las paredes

    Todas las caras verticales de los brushes seleccionados reciben trim. Para caras sueltas, haz Shift+clic en ellas, y Ctrl+Shift+clic añade o quita más.

3. Abre el diálogo

    Elige *Brush > Añadir trim…*. Empieza con el material actual, y **Usar actual** vuelve a él más tarde. Abajo indica cuántas caras encontró.

4. Ajusta la tira

    Elige en qué **Aristas** se pone: abajo, arriba, lados o todas. **Altura** es cuánto recorre la tira sobre la cara, **Profundidad** cuánto sobresale, y **Retranqueo** deja un hueco entre el borde y la tira.

    ![El diálogo Añadir trim con Aristas en Abajo, Altura 6 u, Profundidad 2 u, Retranqueo 0 u, showcase/planks_dark como material, Solo donde hay un suelo bajo la arista marcado, Colisión desactivada y 20 caras seleccionadas](../assets/trim/add-trim-dialog.png)

5. Haz clic en **Añadir trim**

    La barra de estado indica cuántas tiras creó.

Las tiras acaban en un `func_detail_illusionary` mientras **Colisión** está desactivada, que es lo predeterminado, para que los jugadores no se atasquen en ellas. Márcala, o usa un juego sin esa entidad, y acaban en un grupo normal.

Donde se encuentran dos caras con trim, una tira llega hasta la esquina y la otra se detiene en ella, así que las esquinas cierran sin que dos tiras se solapen. Alrededor del extremo de una pared exenta o de un vano de puerta la tira da la vuelta. Una tira que acabaría dentro de otro brush, como en los extremos de un dintel entre dos trozos de pared, se omite.

**Solo donde hay un suelo bajo la arista** limita el trim inferior a las aristas que se apoyan en algo, así que la cara inferior de un dintel sobre un vano de puerta no recibe zócalo.

> [!TIP]
> El trim son brushes normales. Cambia la textura, elimina o remodela tiras sueltas después como con cualquier otro brush.

<div class="td-mcp" markdown>

## MCP { #mcp }

La herramienta `trim` toma `faces` como `[[brush, face], ...]`, o los `ids` de brushes, o la selección, además de `edges`, `height`, `depth`, `inset`, `material`, `on_floor` y `collision`. Devuelve el nuevo grupo o entidad y cuántas tiras creó.

</div>
