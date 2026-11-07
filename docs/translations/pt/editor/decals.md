---
source: 2bf892f3f08e81fde13de0d18bb8d2c0a06c8753
---

# Decals { #decals }

Os decals põem sujeira, manchas, rachaduras, pôsteres e placas numa superfície. Eles são desenhados um fio mais perto da câmera do que a parede atrás deles e misturados por cima dela, então nunca piscam contra a parede nem uns contra os outros.

## Posicionar um decal { #placing-a-decal }

1. Ache o material

    Procure-o no painel Materiais, a caixa de busca no canto superior esquerdo ajuda.

2. Segure Alt e arraste-o para uma face

    O cartão junto ao cursor lembra que Alt solta um decal.

    ![Arrastando night/shop_sign sobre uma parede de tijolos, o cartão junto ao cursor diz Solte numa face, Shift cobre o brush todo, Alt solta um decal](../assets/decals/decal-alt-drop.png)

3. Solte

    Você deve ver o decal cair onde o soltou, como um quadrado de um metro, 32 por 32 unidades, voltado para fora da face.

    ![O decal da placa da loja na parede de tijolos](../assets/decals/decal-on-wall.png)

Um decal é uma malha normal, então mova, gire e escale como uma. Com ele selecionado, o Inspetor mostra **Decal (misturado, face dupla)** marcado, e os campos de posição e tamanho para posicioná-lo com exatidão.

![O Inspetor com o decal selecionado, Decal (misturado, face dupla) marcado e o tamanho 32 por 32](../assets/decals/decal-inspector.png)

Essa mesma caixa transforma qualquer malha fina em decal, para uma com contorno próprio.

Bordas suaves continuam suaves. Para um corte seco, use um material no modo alpha scissor, veja [Transparência](../godot/materials.md#transparency).

## Por que não um brush fino { #why-not-a-thin-brush }

> [!WARNING]
> Não imite decals com brushes ou malhas finas a poucos milímetros de uma parede. Eles piscam.

De perto um brush fino parece bom. Dê um passo para trás e ele começa a piscar através da parede em listras que se arrastam enquanto você se move. Isso se chama z-fighting.

A GPU descobre qual superfície é a mais próxima em cada pixel comparando suas profundidades, e guarda a profundidade em um número limitado de passos. Os passos ficam mais grossos quanto mais longe uma superfície está e quanto mais rente você olha ao longo dela. A alguns metros, o brush e a parede arredondam para o mesmo passo, então cada pixel escolhe um dos dois quase ao acaso, e a escolha muda toda vez que a câmera se move. Afastar o brush da parede só empurra a piscada mais para longe, e de perto ele então flutua visivelmente na frente da parede.

Um decal não tem esse problema. Quando o Godot constrói o mapa, ele puxa cada decal na direção da câmera em 0,05 por cento da distância dele, meio milímetro a 1 metro e 5 centímetros a 100 metros, então ele fica na frente da parede em qualquer alcance. O puxão corre ao longo da sua linha de visão, então o decal não se desloca na tela. Ele também não grava profundidade própria, então decals que se sobrepõem se misturam uns por cima dos outros em vez de brigar.

Uma malha fina serve quando você marca **Decal (misturado, face dupla)** nela, isso a torna um decal de verdade.

## Decals projetados { #projected-decals }

Um decal comum é plano. Para uma mancha que contorna um degrau, um cano ou um terreno irregular, use um decal projetado. É uma entidade que lança um material sobre tudo que está dentro da sua caixa, props incluídos. Custa mais para desenhar, então fique com os decals comuns em superfícies planas.

Ele vem com o pacote de entidades de gameplay como `infodecal`. [Decals projetados](projected-decals.md) mostra como configurá-lo, posicioná-lo e o que observar.

<div class="td-mcp" markdown>

## MCP { #mcp }

Pelo [servidor MCP](../mcp.md), posicione um decal com `run_action` `create_decal`, passando `material`, `at`, `normal` e `size` como `[width, height]`.

</div>
