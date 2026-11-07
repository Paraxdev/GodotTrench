---
source: be4d0912557053b84612829a7948190773c48b4d
---

# Ilha no mar 1: terreno { #sea-island-1-terrain }

Uma ilha grande com dois morros, penhascos, uma praia, trilhas, mata e um píer, feita em cinco partes. Os números são os
usados nas imagens, em unidades de mapa a 32 por metro.

![A ilha pronta vista do sul](../assets/sea-island/island-finished.jpg)

## Crie o terreno { #create-the-terrain }

Vá em *Terreno > Criar terreno…* na barra de menus.

![O menu Terreno com Criar terreno… sob o mouse](../assets/sea-island/menu-create-terrain.png)

Preencha a janela assim:

| Ajuste | Valor e motivo |
| --- | --- |
| **Resolução**, **Tamanho da célula** | 257 x 257 pontos, a 64 unidades um do outro. A linha **Tamanho** embaixo mostra o que você ganha, 16384 unidades (512 m) |
| **Forma** | Ilha, **Altura** 1600, **Tamanho dos detalhes** 3000, **Erosão** 20, **Semente** 1. A ilha sobe no meio e desce em direção às bordas. O tamanho dos detalhes é a largura das maiores elevações, e a erosão deixa as encostas íngremes desabarem para parecerem desgastadas |
| **Camada base**, **Camada de declive**, **Camada de pico**, **Camada baixa** | `showcase/grass`, `showcase/cliff`, `showcase/dirt`, `showcase/sand`. Ladrilho 256 nas duas primeiras, 192 nas outras. A base cobre tudo o que você não pintar, as outras servem para terreno íngreme, alto e baixo |
| **Centro** | Desmarque **Cursor 3D**, depois `0 -576 0` |
| **Pintar camadas por declive e altura** | Desmarcado, a parte 3 pinta à mão |

(O valor de ladrilho é quantas unidades uma repetição da textura cobre.)

![A janela Criar terreno preenchida: 257 x 257, tamanho da célula 64, Ilha, altura 1600, tamanho dos detalhes 3000, erosão 20, semente 1, as quatro camadas showcase, centro 0 -576 0 e Pintar camadas por declive e altura desmarcado](../assets/sea-island/create-terrain-dialog.png)

Clique em **Criar** embaixo. Você deve ver uma ilha verde aparecer, já selecionada, e o editor muda para a ferramenta
Esculpir por você.

O mar ficará na altura 0, então baixar o terreno em 576 inunda o anel externo e deixa o meio seco. As partes seguintes se referem às camadas pelo
número do slot: 0 grama, 1 penhasco, 2 terra, 3 areia.

## Adicione o mar { #add-the-sea }

O mar é um brush plano onde o jogador pode entrar andando.

1. Desenhe uma caixa enorme

    Aperte **Q** para a ferramenta Selecionar e **Escape** para o terreno não ficar mais selecionado, senão o seu arrasto esculpe ou
    move o terreno. Depois arraste na vista Topo para desenhar uma caixa muitas vezes maior que o terreno, ou a borda dela aparece no
    horizonte.

2. Achate-a

    Na vista Frente, deixe-a com 32 unidades de espessura e o topo na altura 0.

3. Transforme em água

    Com a caixa selecionada, clique em `showcase/water` no painel Materiais. Depois Shift+clique na água na vista 3D para
    selecionar só a face de cima, e defina *escala* como 8 e 8 na seção *Alinhamento* do Inspetor. Assim a
    textura de água não se repete a cada poucos metros.

    ![O Inspetor com 1 face selecionada, material showcase/water e a escala do Alinhamento em 8,00 e 8,00](../assets/sea-island/inspector-alignment-scale.png)

4. Deixe o jogador entrar na água

    Clique uma vez na água, longe da ilha, para a caixa inteira ficar selecionada de novo em vez de só a face de cima. Depois
    clique com o botão direito e escolha *Criar entidade de brush > func_illusionary*. Ele é desenhado sem colisão, então o jogador pode
    entrar andando na água.

Agora você deve ter uma ilha no mar, pronta para esculpir:

![A ilha gerada no mar, antes de esculpir](../assets/sea-island/fresh-island.jpg)

Próximo: [Ilha no mar 2: esculpir](sea-island-sculpt.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

O [servidor MCP](../mcp.md) permite que agentes e scripts controlem o editor.
[examples/mcp/sea_island.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/sea_island.json) é um
script MCP que monta a mesma ilha, então você pode reproduzi-lo para conferir um valor.

</div>
