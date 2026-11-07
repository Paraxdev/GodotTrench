---
source: 19de62e149bf31bc7f0453a04737b8fbd5510db4
---

# Botão abre uma porta { #button-opens-a-door }

Um botão na parede que abre uma porta deslizante quando o jogador o aperta. É o pedacinho mais simples e útil de I/O de entidades:
o botão dispara uma saída chamada `pressed`, e uma conexão transforma isso numa chamada da entrada `open` da porta.

> [!TIP]
> Se saídas, entradas e `targetname` são novidade para você, leia antes [Como funciona o I/O de entidades](../gameplay/io.md). Leva
> só alguns minutos e esta página se apoia nele.

## Ligue tudo { #wire-it }

`func_door` e `func_button` vêm com o pacote de entidades de gameplay. Se o seu projeto ainda não o tem, use
*Gameplay > Instalar entidades de gameplay*, ou **Instalar entidades de gameplay** na janela *Configurar este projeto*
(*Godot > Instalar ou atualizar addon…*).

1. Faça a porta

    Desenhe um brush do tamanho de uma porta em um vão, [Primeiros passos](../getting-started.md#4-build-a-room) mostra como abrir um.
    Clique com o botão direito no brush e escolha *Criar entidade de brush > func_door*.

    ![O menu do botão direito sobre um brush laranja de porta, com Criar entidade de brush aberto e func_door destacado](../assets/button-door/create-brush-entity-func-door.png)

2. Dê um nome a ela

    No Inspetor, à direita, digite `main_door` em `targetname`. O nome é como as outras entidades encontram a porta.
    Deixe `travel` em `0 64 0`, assim a porta desliza 64 unidades (2 m) para cima quando abre.

    ![O Inspetor do func_door com targetname main_door e travel 0 64 0](../assets/button-door/inspector-targetname.png)

3. Faça o botão

    Desenhe um brush pequeno na parede ao lado da porta e transforme-o em um `func_button` do mesmo jeito.

4. Selecione os dois

    Clique no botão, depois Ctrl+clique na porta, em uma vista ou no Outliner, à esquerda. Tem paredes no caminho? Digite `func`
    na caixa *Filtro* no topo do Outliner, e só esses dois ficam listados.

    ![O Outliner filtrado por func, com func_door (main_door) e func_button selecionados](../assets/button-door/outliner-filter-func.png)

5. Vincule os dois

    Vá em *Gameplay > Lógica > Vincular duas entidades selecionadas…*, ou aperte **Vincular…** no topo do Inspetor.

    ![O menu Gameplay com Lógica aberto e Vincular duas entidades selecionadas… destacado](../assets/button-door/menu-link-two-selected.png)

    A entidade que dispara precisa ficar à esquerda, aqui o botão. A janela coloca à esquerda a que você criou primeiro, que é a
    porta, então aperte o botãozinho de troca entre os dois nomes. Escolha `pressed` em *Quando* e `open` em *chamar*, e
    clique em **Vincular**.

    ![A janela Vincular entidades com func_button à esquerda, func_door (main_door) à direita, Quando pressed e chamar open](../assets/button-door/link-dialog.png)

A barra de status, embaixo à esquerda, deve dizer agora `pressed > main_door.open`. Leia como "quando pressed, chame open em
main_door".

![A barra de status dizendo pressed > main_door.open](../assets/button-door/status-linked.png)

Selecione o botão sozinho e a conexão aparece em *Saídas* no Inspetor dele, onde você pode mudá-la ou removê-la depois.

![A seção Saídas do func_button: saída pressed, alvo main_door, entrada open](../assets/button-door/inspector-button-output.png)

> [!TIP]
> *Gameplay > Portas e móveis*, que também fica na seção *Gameplay* do Inspetor, transforma o brush selecionado em uma porta
> de dobradiça ou deslizante com um clique. Ele dá um nome à porta, define como ela abre e coloca na frente dela um
> `trigger_multiple` que abre a porta quando o jogador chega perto. Apague esse trigger se só o botão deve abrir a porta.
>
> ![O menu Gameplay com Portas e móveis aberto: porta de dobradiça com dobradiça à esquerda ou à direita, porta deslizante para cima ou para o lado, e Elevador, Plataforma móvel](../assets/button-door/menu-doors-and-movers.png)

## Deixe o jogador apertar { #let-the-player-press-it }

O addon não tem um sistema de interação próprio, já que cada jogo trata o "usar" de um jeito. Portas e botões têm um método
`use(activator)` e o script do seu jogador o chama. O jogador da demo faz isso com **E**: ele lança um raio a partir da
câmera e chama `use` na primeira coisa que atinge que tenha um
([godot/demo/player.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd)):

```gdscript
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
```

O jogador se passa como ativador, então `pressed(activator)` leva o jogador junto. Saídas posteriores da cadeia podem ter como alvo
`!activator` para agir em quem apertou o botão.

## Variações { #variations }

| Você quer | O que mudar |
| --- | --- |
| A porta só abre pelo botão | Desmarque `interact` da porta. Senão apertar **E** na porta também a abre |
| A porta fecha sozinha | Defina o `wait` da porta como os segundos que ela fica aberta. O padrão, -1, a mantém aberta |
| Porta trancada | Marque `locked`. A porta ignora `open` e `use` e dispara `locked_use`, até uma saída chamar a entrada `unlock` dela, por exemplo de um item de chave |
| Um botão, várias portas | Chame as portas de `gate_1`, `gate_2` e assim por diante e use `gate_*` como alvo. Um `*` no final casa com todo nome que começa com `gate_` |
| Porta giratória | Use `func_door_rotating` no lugar e arraste a alça da dobradiça na vista até onde a dobradiça deve ficar |

## A mesma ligação em código { #the-same-wiring-in-code }

Prefere conectar as coisas em GDScript a fazer no editor? Isto faz o mesmo que a conexão acima:

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in TrenchDotIO.find_targets(button, "main_door", activator):
		TrenchDotIO.invoke(door, &"open", "", activator))
```

Conectar `pressed` direto a `door.open` falha, porque `pressed` passa um argumento e `open()` não recebe nenhum.
`invoke` ajusta os argumentos ao método, do mesmo jeito que as conexões do mapa fazem.
