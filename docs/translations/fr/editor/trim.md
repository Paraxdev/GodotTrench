---
source: 3cc68efb1c3e922c9ae3811f442a5d5e642414b6
---

# Trim { #trim }

*Brush > Ajouter un trim…* pose des bandes de brush le long des arêtes des faces, en saillie. Au pied des murs ce sont
des plinthes, en haut des corniches, autour d'un sol une bordure, autour d'un bassin son rebord. Voici une pièce
simple avant et après un clic :

![Une pièce enduite avec une porte et ses murs sélectionnés, sans trim pour l'instant](../assets/trim/room-before.png)

![La même pièce avec des plinthes sombres au pied des murs, qui montent jusqu'à la porte](../assets/trim/room-after.png)

1. (Facultatif) Choisir le matériau du trim

    Appuyez sur Esc pour ne rien avoir de sélectionné, puis cliquez sur le matériau dans le panneau Matériaux. Il
    devient le matériau actuel sans modifier la carte.

2. Sélectionner les murs

    Chaque face verticale des brushes sélectionnés reçoit du trim. Pour des faces isolées, faites plutôt Shift+clic dessus,
    Ctrl+Shift+clic en ajoute ou en retire d'autres.

3. Ouvrir la boîte de dialogue

    Choisissez *Brush > Ajouter un trim…*. Elle démarre avec le matériau actuel, **Utiliser l'actuel** y revient plus
    tard. En bas, elle indique combien de faces elle a trouvées.

4. Régler la bande

    Choisissez quelles **Arêtes** en reçoivent : bas, haut, côtés ou toutes. **Hauteur** est la distance sur laquelle la
    bande court sur la face, **Profondeur** la distance dont elle dépasse, et **Retrait** laisse un espace entre l'arête
    et la bande.

    ![La boîte de dialogue Ajouter un trim avec Arêtes sur bas, Hauteur 6 u, Profondeur 2 u, Retrait 0 u, showcase/planks_dark comme matériau, Seulement là où un sol est sous l'arête coché, Collision désactivée et 20 faces sélectionnées](../assets/trim/add-trim-dialog.png)

5. Cliquer sur **Ajouter un trim**

    La barre d'état indique combien de bandes elle a créées.

Les bandes atterrissent dans un `func_detail_illusionary` tant que **Collision** est désactivée, ce qui est le réglage
par défaut, pour que les joueurs ne s'y accrochent pas. Cochez-la, ou utilisez un jeu sans cette entité, et elles
atterrissent dans un simple groupe à la place.

Là où deux faces avec trim se rejoignent, une bande va jusqu'au coin et l'autre s'y arrête, si bien que les coins se
ferment sans que deux bandes se chevauchent. Autour du bout d'un mur isolé ou d'une porte, la bande fait le tour. Une
bande qui finirait à l'intérieur d'un autre brush, comme aux extrémités d'un linteau entre deux morceaux de mur, est
omise.

**Seulement là où un sol est sous l'arête** limite le trim du bas aux arêtes qui reposent sur quelque chose, si bien que
le dessous d'un linteau au-dessus d'une porte ne reçoit pas de plinthe.

> [!TIP]
> Le trim, ce sont de simples brushes. Retexturez, supprimez ou remodelez ensuite chaque bande comme n'importe quel
> autre brush.

<div class="td-mcp" markdown>

## MCP { #mcp }

L'outil `trim` prend `faces` sous la forme `[[brush, face], ...]`, ou des `ids` de brush, ou la sélection, plus `edges`,
`height`, `depth`, `inset`, `material`, `on_floor` et `collision`. Il renvoie le nouveau groupe ou la nouvelle entité et
le nombre de bandes créées.

</div>
