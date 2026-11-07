---
source: b549cb4d1d3fd2ab8e0ac144848e50eac2b168fc
---

# Organizando um mapa { #organizing-a-map }

Quando um mapa cresce, você quer separá-lo em partes, achar as coisas de novo e manter num só lugar as peças que reaproveita. Quase tudo isso começa no Outliner, o painel à esquerda.

## Camadas { #layers }

O Outliner mostra o mapa como uma árvore de camadas, grupos e objetos. As camadas deixam você dividir um mapa em partes, por exemplo um prédio por camada, para poder ocultar ou travar uma parte enquanto trabalha no resto.

Clique em **Adicionar camada** no topo do Outliner para criar uma nova, ela vira a camada atual na hora. Os objetos novos vão para a camada atual, a marcada com *(atual)*, e clicar numa camada a torna atual. O olho na frente de uma linha a oculta, o cadeado impede que ela seja selecionada ou editada.

Passe o mouse sobre uma linha para ver os materiais, o tamanho e os limites dela. O objeto fica contornado em todas as vistas ao mesmo tempo, então dá para ver qual parede é o `brush7`.

![O Outliner com a camada House aberta e a linha brush7 sob o mouse, uma dica mostra seu material, tamanho e limites enquanto as vistas 3D e Frente contornam a parede dos fundos](../assets/organizing/outliner-hover.png)

Brushes e malhas são listados por tipo e id, como `brush12` ou `mesh40`, para que dê para distinguir as paredes que um Esvaziar ou um Subtrair deixa para trás. O id continua o mesmo quando o objeto se move e entre os salvamentos.

### Dando nomes às coisas { #naming-things }

Selecione um objeto e aperte F2, ou clique com o botão direito nele no Outliner ou numa vista e escolha *Renomear*. A linha vira um campo de texto. Enter mantém o nome, Escape cancela, e um nome vazio traz de volta o padrão.

![Uma linha de brush no Outliner transformada em campo de texto com "front wall" digitado](../assets/organizing/outliner-rename.png)

Os nomes são salvos com o mapa. As entidades guardam o `targetname` separado desse nome, então renomear nunca muda o que uma saída alcança. As peças que um Clip, uma operação de CSG ou uma conversão deixa para trás mantêm o nome, e dar nome a um objeto dentro de um grupo vinculado dá nome a ele em todas as cópias.

(Camadas, grupos e conjuntos de scatter sempre mantêm um nome, então para eles um nome vazio não muda nada. Renomear um conjunto de scatter também renomeia a camada que a ferramenta Scatter criou para ele.)

### Escolhendo objetos no Outliner { #picking-objects-in-the-outliner }

Ctrl+clique numa linha para adicioná-la à seleção ou tirá-la. Shift+clique seleciona todas as linhas entre a última em que você clicou e esta, e Ctrl+Shift+clique adiciona esse intervalo ao que já está selecionado. O Outliner costuma ser o lugar mais fácil para escolher objetos que ficam escondidos atrás de outros nas vistas.

### Mantendo uma camada fora do Godot { #keeping-a-layer-out-of-godot }

Para geometria de referência que nunca deve chegar ao jogo, clique com o botão direito na camada e escolha *Alternar omitir na exportação*. A camada passa a mostrar *[omitida]* depois do nome. Ocultar não basta, o Godot ainda constrói os objetos ocultos.

![O menu do botão direito da camada de referência Blockout com Alternar omitir na exportação sob o mouse](../assets/organizing/layer-menu-omit.png)

## Grupos { #groups }

Um grupo mantém objetos juntos para que sejam selecionados e movidos como um só, como os brushes de uma mesa. Tudo sobre grupos fica em *Editar > Grupos*.

![O menu Editar com Grupos aberto e Duplicar vinculado sob o mouse](../assets/organizing/menu-groups.png)

| Entrada | Resultado |
| --- | --- |
| Ctrl+G, Ctrl+Shift+G | Agrupar a seleção, desagrupá-la |
| Clique | Seleciona o objeto dentro do grupo |
| Clique duplo | Seleciona o grupo inteiro |

**Duplicar vinculado** (Ctrl+Shift+D) faz uma cópia que continua em sincronia com a original, então consertar um batente de janela conserta todos. As cópias vinculadas mostram *(linked)* no Outliner, e um clique em uma seleciona o grupo inteiro, já que as cópias são editadas como uma só. *Editar > Grupos > Desvincular grupos* quebra o vínculo quando uma cópia precisa ser diferente.

## Prefabs { #prefabs }

Para peças que você reaproveita entre mapas, selecione-as e escolha *Editar > Prefabs > Criar a partir da seleção…*. Isso salva a seleção como um `.tdm` próprio e coloca uma instância no lugar dela. Toda instância se atualiza quando o arquivo do prefab muda.

![O menu Editar com Prefabs aberto e Criar a partir da seleção sob o mouse](../assets/organizing/menu-prefabs.png)

> [!NOTE]
> Salve o mapa uma vez antes de criar um prefab. Os caminhos dos prefabs são guardados relativos ao mapa, então o editor se recusa até o mapa ter um arquivo.

Para peças que você cola uma vez e depois molda livremente, use a [biblioteca de prefabs](prefab-library.md).

Colocar um prefab duas vezes dá a duas entidades o mesmo targetname. O fixup de uma instância resolve isso: com um fixup `p1`, todo nome dentro dela ganha esse prefixo, então `door` vira `p1-door` e cada cópia mantém sua ligação separada. O arquivo do mapa e a construção no Godot conhecem o fixup, mas o editor ainda não tem um campo para defini-lo.

## Cordon { #cordon }

Um cordon reduz o editor a uma parte de um mapa grande.

1. Selecione o que você quer trabalhar.
2. Escolha *Exibir > Cordon > Definir cordon a partir da seleção*, ou clique em **Cordon** no Inspetor.

    ![O menu Exibir com Cordon aberto e Definir cordon a partir da seleção sob o mouse](../assets/organizing/menu-cordon.png)

3. Tudo que não toca a caixa amarela em volta da sua seleção agora fica oculto e não pode ser selecionado, e a barra de status diz *cordon*.

    ![Antes: o mapa inteiro com a casa, o gramado, um muro de jardim, um pilar e um bloco alto de referência. Depois: só a casa dentro de uma caixa amarela e o gramado que a toca](../assets/organizing/cordon-before-after.png)

*Exibir > Cordon > Cordon ativado* o desliga e liga de novo, e *Limpar cordon* o remove. O cordon é salvo com o mapa.

Ele só muda o que o editor mostra, o Godot ainda constrói o mapa inteiro. Para entregar uma área a outra ferramenta, *Arquivo > Exportar > .map, só cordon…* escreve apenas os objetos dentro dele.

## Problemas { #issues }

O painel Problemas, a aba ao lado do Outliner e do Histórico, confere o mapa enquanto você trabalha. Ele lista problemas que quebrariam ou ficariam errados no Godot, como brushes sem volume, faces sem material ou saídas voltadas para um targetname que nada tem.

![O painel Problemas com um erro de alvo ausente, três avisos de brush duplicado, entidade sem definição e duas faces com z-fighting, e uma linha de informação para um grupo vazio, a maioria com um botão Corrigir](../assets/organizing/issues-panel.png)

Clique num problema para selecionar e enquadrar o objeto. Muitos têm um botão **Corrigir**, e **Corrigir tudo** no cabeçalho corrige todos os que conseguir.

Um aviso de que uma face se sobrepõe a uma face de outro brush no mesmo plano significa que as duas vão piscar no Godot. Tire uma das faces do plano ou apare a sobreposição.

Um aviso de que um modelo não existe significa que o `model` de um prop aponta para um arquivo que não está lá, então o prop é construído sem malha. Quando existe um arquivo com o mesmo nome e outra extensão, `chair.gltf` para `chair.glb`, o aviso o nomeia.

Um aviso de que algo executa GDScript a partir do mapa significa que o mapa carrega código próprio, que roda assim que a cena é jogada no Godot. Você recebe um para cada `logic_script` com um `source` ou `expression`, para cada saída ou `logic_call` que pode entregar código a um deles, como uma saída que envia `source` para ele, e para cada prefab que contém qualquer um destes. Leia-os antes de jogar um mapa que outra pessoa fez.

Os avisos que começam com "Godot" vêm da última construção no Godot, depois de um salvamento ou de *Construir no Godot*: as classes que ele construiu sem a definição delas, e os outros avisos dessa construção. Veja [Entidades que não fazem nada](../godot/entities-troubleshooting.md).

## Histórico { #history }

O painel Histórico lista cada edição, a mais nova no topo. Clique numa entrada mais antiga para desfazer até ela, incluindo essa entrada. As edições desfeitas ficam esmaecidas acima de *atual*, clique numa para refazer até ela.

![O painel Histórico com Renomear e Agrupar esmaecidos acima do marcador atual e a entrada Esvaziar sob o mouse](../assets/organizing/history-panel.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

Um agente ou script que controla o editor pelo [servidor MCP](../mcp.md) executa as mesmas verificações do painel Problemas com a ferramenta `validate_map`, que as nomeia por código, como `coplanar_faces` e `missing_model`. A ação `insert_prefab` do `run_action` aceita um `fixup`, então um agente pode dar a uma instância o fixup dela ao colocá-la.

</div>
