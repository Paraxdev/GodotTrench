---
source: 2de051fc5fd93ade8349b194b97a6c34a8399111
---

# Decals projetados { #projected-decals }

Um decal projetado é uma caixa que projeta um material sobre tudo o que está dentro dela. Ao contrário de um [decal comum](decals.md), que é uma folha plana, ele acompanha o que atinge, então uma mancha pode escorrer por um degrau, contornar um cano ou ficar deitada num terreno irregular. Props e modelos dentro da caixa também recebem o decal.

Na cena construída ele é um nó `Decal` do Godot, e custa mais para desenhar do que um decal comum. Use-o onde uma folha plana não resolve.

> [!WARNING]
> Decals projetados pesam mais na GPU do que [decals comuns](decals.md). O Godot calcula o decal para cada pixel que a caixa cobre na tela, então muitas caixas, ou caixas grandes, deixam o jogo mais lento, principalmente em placas de vídeo mais fracas. Use um decal comum numa superfície plana, deixe a caixa tão pequena quanto o trabalho permitir e reserve os decals projetados para os lugares que precisam do contorno.

## Adicionando ao seu projeto { #adding-it-to-your-project }

O decal projetado vem com o [pacote de entidades de gameplay](../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) como `infodecal`, então configurá-lo é instalar o pacote.

1. Instale ou atualize o pacote

    Clique em **Instalar entidades de gameplay** na janela *Configurar este projeto*, ou em **Atualizar entidades de gameplay** no menu Gameplay quando o pacote já está no seu projeto. [Instalando e atualizando o pacote](../godot/project-setup.md#installing-and-updating-the-pack) mostra o passo a passo.

2. Confira se o editor o enxerga

    Você deve ver um cartão `infodecal` na aba **Entidades**, ao lado do Inspetor. Digite `infodecal` na caixa de busca no topo da aba para achá-lo rápido.

    ![A aba Entidades buscando infodecal, o cartão com o cursor em cima e sua dica dizendo Decal projetado](../assets/projected-decals/entities-infodecal-card.png)

> [!NOTE]
> Se você criou sua própria definição de `infodecal` antes de o pacote ter uma, tire-a do seu FGD depois de atualizar. Senão a classe fica definida duas vezes, e o editor avisa.

> [!TIP]
> Não usa o pacote? Copie a definição dele, [`infodecal.tres`](https://github.com/Paraxdev/trenchdot_func/blob/main/gameplay_pack/definitions/infodecal.tres), para o seu projeto e adicione-a a *Entity Definitions* do seu próprio arquivo FGD, veja [Entidades personalizadas](../gameplay/custom-entities.md#making-it-placeable). Mantenha *Node Class* em `Decal`, o script precisa disso e é assim que o editor sabe tratar a entidade como um decal.

## Posicionando um decal { #placing-one }

1. Arraste o cartão para uma superfície

    Abra a aba **Entidades** ao lado do Inspetor e arraste o cartão `infodecal` para uma vista, do mesmo jeito que você posiciona qualquer entidade (veja [Posicionando entidades](../gameplay/placing.md#placing-entities)). Ele cai onde o ponteiro encontra a primeira superfície sólida, virado para projetar para dentro dela. Uma caixa mostra o espaço que ele cobre, metade na frente da superfície e metade atrás.

    ![Um infodecal na quina de um degrau de tijolo na vista 3D, a caixa passando por cima e descendo pela frente, com o material da placa em prévia dentro dela](../assets/projected-decals/decal-box-on-step.png)

2. Escolha o material

    No Inspetor, digite o nome de um material em `material`, o mesmo nome que uma face usa, como `base/stain`. A cor, o mapa normal e o brilho dele passam junto.

    ![O Inspetor com o infodecal selecionado, night/shop_sign em material e 128 64 64 em size](../assets/projected-decals/decal-inspector-material.png)

3. Defina o tamanho

    `size` recebe três números em unidades de mapa: largura, profundidade e altura. A largura e a altura são o retalho que ele cobre na superfície, a profundidade é quanto a caixa entra e sai dela. O padrão é 64 32 64, um quadrado de dois metros que alcança meio metro para cada lado. Deixe no padrão e o retalho passa a usar o [tamanho de textura](../godot/materials.md#texture-size) do material, se ele tiver um, no editor e no Godot.

4. (Opcional) Aponte-o

    Gire-o como qualquer outra entidade. Ele sempre projeta ao longo do próprio eixo para baixo, -Y local, que o arraste apontou para dentro da superfície.

5. Construa o mapa no Godot

    O material aparece em toda superfície dentro da caixa quando o Godot constrói o mapa, veja [Construindo mapas](../godot/building.md).

    O editor mostra só uma prévia plana do material dentro da caixa, o Godot desenha a projeção de verdade, curvando a placa sobre a quina do degrau e descendo pela frente dele.

    ![A mesma placa num jogo Godot em execução, curvada sobre a quina do degrau de tijolo e na face da frente dele](../assets/projected-decals/decal-in-game.png)

## Chaves { #keys }

| Chave | O que faz |
| --- | --- |
| `material` | O material a projetar, um nome como `base/stain` ou um caminho `res://`. São usadas as texturas de albedo, normal, ORM e emissão dele, e a cor de albedo |
| `texture` | Um caminho `res://` para uma imagem, projetada quando `material` está vazio |
| `normal_texture` | Um caminho `res://` para um mapa normal, para usar com `texture`. O mapa normal do próprio material o substitui |
| `size` | Largura, profundidade e altura da caixa em unidades de mapa, 64 32 64 a menos que você mude |
| `modulate` | Uma cor pela qual a imagem projetada é multiplicada |

## Cuidados { #things-to-watch-out-for }

| Situação | O que acontece |
| --- | --- |
| O projeto usa o renderizador Compatibility | O Godot não desenha nenhum decal. Mude *Rendering Method* para Forward+ ou Mobile em *Project Settings > Rendering > Renderer* |
| O projeto usa o renderizador Mobile | Só 8 decals aparecem em cada malha, o resto fica de fora |
| Faces que correm ao longo da projeção | A textura estica em riscos, como na frente de um degrau sob um decal que aponta direto para baixo. Incline a caixa para que ela encontre as duas faces em ângulo |
| Superfícies perto das pontas da caixa | O decal some aos poucos em direção ao topo e à base da caixa, então mantenha a superfície perto do meio, onde o arraste a coloca |
| Algo atravessa a caixa | Também recebe o decal, inclusive um jogador que passe por perto. Deixe a caixa só tão funda quanto a superfície precisa |
| Vidro e recortes | Continuam tão transparentes quanto antes, um decal não preenche partes transparentes |
| Muitos decals grandes | Cada um acrescenta trabalho a todo pixel que cobre, então deixam o jogo lento onde se acumulam |

<div class="td-mcp" markdown>

## MCP { #mcp }

Pelo [servidor MCP](../mcp.md), `run_action` com `action: "place_entities"` e `args: {"classnames": ["infodecal"], "at": [x, y, z], "normal": [x, y, z]}` o posiciona virado para a superfície, como um arraste faz. Defina as chaves dele depois com `update_entity` `properties`.

</div>
