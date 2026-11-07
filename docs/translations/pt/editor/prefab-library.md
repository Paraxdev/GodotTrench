---
source: be5056a246dacf54d3a4aecafff90bff3f0d1aad
---

# Biblioteca de prefabs { #prefab-library }

A aba **Prefabs** embaixo, ao lado de Materiais e Modelos, guarda peças prontas para colar num mapa: escadas, portais, pilares, salas pequenas e detalhes. Pense nas coleções de prefabs de onde os mappers do TrenchBroom copiam, só que embutidas no editor.

![A aba Prefabs com Buscar prefabs, a lista de categorias, o controle de tamanho dos cartões e um cartão para cada peça embutida, a Escada em zigue-zague sob o mouse com sua dica: Escada/Escada em zigue-zague, 192 × 128 × 352 unidades, Integrado](../assets/prefab-library/prefabs-panel.png)

Cada cartão desenha a peça pelo lado que mostra mais: de cima para plantas baixas como uma escada, de frente para paredes como um portal. Mais claro significa mais perto.

## Colando uma peça { #pasting-a-piece }

1. Aponte para o lugar

    Passe o mouse sobre o ponto numa vista onde a peça deve ficar, isso define o cursor 3D.

2. Clique no cartão

    Digite em *Buscar prefabs* ou escolha uma categoria para achá-la, depois clique nela. A peça é colada centralizada nesse ponto e alinhada à grade, como Ctrl+V.

    ![A Escada em zigue-zague colada numa salinha, os degraus nos materiais dev cinzas](../assets/prefab-library/pasted-switchback-stair.png)

Só o ponto na planta baixa segue o cursor. A altura fica como a peça foi salva, então num andar de cima levante-a depois.

A colagem é uma cópia simples sem ligação de volta com a biblioteca, então remodele ou retexturize à vontade. As peças embutidas usam os materiais `dev/` que todo projeto tem, retexturize-as na aba Materiais.

Clique com o botão direito num cartão para *Colar* e *Copiar*. Copiar põe a peça na área de transferência, para colar em outro mapa ou compartilhar como texto.

Para cobrir uma área inteira com peças escolhidas ao acaso, use o [Preenchimento aleatório](random-fill.md).

> [!TIP]
> Uma peça é conteúdo de mapa comum depois de colada. Para algo que deve ficar sincronizado em todo lugar onde é usado, como um caixilho de janela repetido por um prédio, faça uma [instância de prefab](organizing.md#prefabs).

## Suas próprias peças { #your-own-pieces }

1. Selecione os objetos

    Selecione o que deve virar a peça, com um projeto Godot aberto.

2. Adicione a seleção

    Abra **Adicionar a seleção** no topo da aba Prefabs, dê um nome e uma categoria à peça e clique em **Adicionar**.

    ![Adicionar a seleção aberto, com Grand stair como nome, Escada como categoria, Manter posição desmarcado e a dica de Adicionar: Salva a seleção como .tdm em res://prefab_library, com o centro da base na origem](../assets/prefab-library/add-the-selection.png)

3. Ache o cartão novo

    Ele aparece primeiro, com uma moldura azul. Clique com o botão direito nele para *Abrir para editar*, que abre a peça como um mapa, e *Mostrar no gerenciador de arquivos*. Peças embutidas não podem ser editadas.

    ![Menu do botão direito no cartão Grand stair com Colar, Copiar, Abrir para editar sob o mouse e Mostrar no gerenciador de arquivos](../assets/prefab-library/card-menu.png)

A peça é salva como um `.tdm` em `res://prefab_library/<category>/`, movida para que o centro da base da seleção fique na origem. Marque **Manter posição** para deixá-la onde está, para uma peça que você construiu em volta da origem do mapa de propósito, como as peças de um kit de [preenchimento aleatório](random-fill.md).

Todo `.tdm` nessa pasta é uma peça. Salve mapas ali você mesmo com *Arquivo > Salvar como…* e eles aparecem como cartões na hora. Para arquivos que você copiou à mão, ou uma pasta de peças compartilhada entre projetos, aperte o botão ⟳.

## Colando do TrenchBroom { #pasting-from-trenchbroom }

O TrenchDot também cola o que o TrenchBroom põe na área de transferência. Copie brushes no TrenchBroom, ou de uma coleção de prefabs do TrenchBroom na web, e aperte Ctrl+V num mapa. Grupos chegam como grupos, entidades como entidades, e os eixos são girados do mesmo jeito que ao [importar um .map](importing.md). Os nomes de textura são mantidos, as texturas de ferramenta de outros jogos viram as do projeto.

<div class="td-mcp" markdown>

## MCP { #mcp }

A ferramenta `prefab_library` lista peças (`op: list`), cola uma pelo seu `category/name` (`op: paste`, com `origin` ou `offset`), devolve o texto da área de transferência dela (`op: copy`) e adiciona objetos à biblioteca do projeto (`op: add`, a seleção a menos que `ids` sejam dados). `run_action` com `paste` aceita texto da área de transferência do TrenchBroom e também o do TrenchDot.

</div>
