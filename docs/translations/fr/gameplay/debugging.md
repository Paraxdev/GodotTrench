---
source: 425ace012cd6d8410cbf64a89bb7dc31275f8980
---

# Déboguer le câblage { #debugging-wiring }

La plupart des erreurs de câblage échouent en silence : la porte reste simplement fermée. Vérifiez dans cet ordre, de la
vérification la plus rapide à la plus complète.

> [!TIP]
> Une porte ou un trigger qui ne bloque même pas le joueur a été construit sans sa définition, ce qui n'est pas une
> erreur de câblage. Voir [Entités qui ne font rien](../godot/entities-troubleshooting.md).

## 1. Le panneau Problèmes { #1-the-issues-panel }

Ouvrez l'onglet **Problèmes**, à côté d'Outliner et d'Historique sur la gauche. Il signale les sorties qui visent un
targetname qu'aucune entité n'a, et les entrées que la classe de la cible n'a pas. Une faute de frappe dans un targetname
est la raison la plus courante pour qu'une porte reste fermée.

![Le panneau Problèmes avec une erreur : la sortie triggered vise l'entité inconnue « hall_lihgt » sur trigger_once, avec un bouton Corriger](../assets/debugging/issues-panel.png)

Cliquez sur un problème pour sélectionner son entité et la cadrer dans les vues, puis corrigez le nom dans l'Inspecteur.

> [!WARNING]
> **Corriger** sur une cible manquante ne devine pas le bon nom, il supprime la connexion. Pour une faute de
> frappe, corrigez plutôt le nom à la main.

## 2. Le Graphe logique { #2-the-logic-graph }

Dans le [Graphe logique](logic-graph.md), une cible que personne n'a apparaît comme un nœud rouge avec un fil rouge en
tirets, qui dit qu'aucune entité n'a ce nom.

![Le Graphe logique : la sortie triggered de trigger_once suit un fil rouge en tirets jusqu'à un nœud rouge hall_lihgt, aucune entité n'a ce nom](../assets/debugging/graph-missing-target.png)

**Simuler** active une sortie sur le papier et met en évidence toute la chaîne, voir
[Simuler](logic-graph.md#simulate).

## 3. F3 en jeu { #3-f3-in-game }

Ajoutez un nœud `TrenchDotDebugOverlay` à votre scène dans Godot, ou ajoutez-le comme autoload. Lancez le jeu et appuyez
sur F3 : il liste les sorties au moment où elles s'activent et dessine chaque volume `Area3D` avec son targetname, pour
que vous voyiez si un trigger ne s'est jamais activé ou s'est activé sur rien.

Rebasculez-le après avoir créé des triggers, les volumes sont collectés quand il s'active. La touche est la propriété
`toggle_key` de l'overlay.

## 4. Journaliser { #4-logging }

Placez une entité `logic_debug` et cochez sa case `trace_all` dans l'Inspecteur. Chaque sortie qui s'active est alors
affichée dans la sortie de Godot.

Depuis le code, connectez-vous à `TrenchDotIO.events().fired`, émis pour chaque sortie, y compris celles dont la cible n'a
rien résolu.

```gdscript
TrenchDotIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## À quoi ressemble chaque erreur { #what-each-mistake-looks-like }

| Erreur | Symptôme | Correction |
| --- | --- | --- |
| La cible ne résout rien | `[TD I/O] <node>.<output>: target '<name>' matches no node` la première fois qu'elle s'active | Faites correspondre exactement la cible au targetname de l'entité, casse comprise |
| La cible n'a pas cette entrée | `[TD I/O] <node> has no input '<name>'` dans la sortie de Godot | Choisissez une entrée que la classe de la cible a, voir la [Référence des entités](entities/README.md) |
| Sortie que l'entité n'émet jamais | `[TD I/O] <node> has no signal '<output>'` une fois, à la construction de la carte | Choisissez une sortie que la classe a, ou activez-la depuis le code avec `TrenchDotIO.fire_output()` |
| Entité créée ou renommée à l'exécution | Introuvable par son targetname | Appelez `TrenchDotIO.invalidate()`, voir [Changements à l'exécution](parameters.md#runtime-changes) |
