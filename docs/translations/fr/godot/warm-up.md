---
source: 70bacdb1e7200d36894611dfa449da2b79911911
---

# Préchauffage des shaders { #shader-warm-up }

Godot compile les shaders la première fois qu'il dessine quelque chose, donc la première image du jeu saccade. Un
préchauffage fait ce travail derrière votre écran de chargement.

Si votre jeu se résume à la carte, vous n'avez rien à faire. Chaque carte se préchauffe toute seule après sa construction
ou son chargement dans le jeu en cours d'exécution, et émet `warmed_up` quand c'est terminé.

Le préchauffage d'une carte ne couvre pourtant que la carte. Si votre jeu ajoute son propre contenu, comme un joueur, des
ennemis ou des armes, leur première apparition saccade encore. Dans ce cas, décochez **Warm Up Shaders** sur le
`FuncGodotMap` dans l'Inspecteur de Godot, et préchauffez tout en une seule passe une fois que tout est en place :

```gdscript
loading.show()
await build_maps()
spawn_player()
await TrenchDotWarmUp.warm_shaders([get_tree().current_scene, preload("res://enemies/crawler.tscn")]).finished
loading.hide()
```

`warm_shaders` accepte des nœuds, des scènes, des maillages et des matériaux. Passez-lui les scènes qui ne sont pas encore
instanciées, comme l'ennemi ci-dessus, pour qu'elles soient prêtes quand elles apparaîtront. Il ne dessine rien à l'écran
et n'exécute aucun de leurs scripts. Connectez son signal `progress(done, total)` pour animer une
barre de chargement.

Sur un jeu de test avec cinq cartes, la première image du jeu est passée de 360 ms à 22 ms, pour une demi-seconde de
préchauffage.

> [!TIP]
> Activez aussi **Shader Baker** dans vos préréglages d'export, pour que les joueurs évitent de compiler les shaders au
> premier lancement.
