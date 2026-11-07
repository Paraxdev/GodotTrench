---
source: 70bacdb1e7200d36894611dfa449da2b79911911
---

# Aquecimento de shaders { #shader-warm-up }

O Godot compila os shaders na primeira vez que desenha algo, então o primeiro quadro do jogo engasga. Um aquecimento faz esse trabalho
atrás da sua tela de carregamento.

Se o seu jogo é só o mapa, você não precisa fazer nada. Todo mapa se aquece depois de construído ou carregado no
jogo em execução e emite `warmed_up` quando termina.

O aquecimento próprio de um mapa cobre só o mapa, porém. Se o seu jogo adiciona conteúdo próprio, como jogador, inimigos ou armas,
a primeira aparição deles ainda engasga. Nesse caso, desmarque **Warm Up Shaders** no `FuncGodotMap`, no Inspetor do Godot,
e aqueça tudo de uma vez quando tudo estiver dentro:

```gdscript
loading.show()
await build_maps()
spawn_player()
await TrenchDotWarmUp.warm_shaders([get_tree().current_scene, preload("res://enemies/crawler.tscn")]).finished
loading.hide()
```

`warm_shaders` aceita nós, cenas, malhas e materiais. Passe cenas que ainda não foram instanciadas, como o inimigo acima, para que
estejam prontas quando aparecerem. Ele não desenha nada na tela e não roda nenhum script delas. Conecte o sinal
`progress(done, total)` dele para alimentar uma barra de carregamento.

Num jogo de teste com cinco mapas, ele levou o primeiro quadro do jogo de 360 ms para 22 ms, por meio segundo de aquecimento.

> [!TIP]
> Ligue também o **Shader Baker** nas suas predefinições de exportação, para que os jogadores pulem a compilação de shaders na primeira execução.
