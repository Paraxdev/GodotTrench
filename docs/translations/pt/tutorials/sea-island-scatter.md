---
source: 4d58791b780474701e360cec324c41c44739f30d
---

# Ilha no mar 4: scatter { #sea-island-4-scatter }

O scatter coloca um monte de árvores, arbustos, pedras ou grama com um brush em vez de uma por uma. Cada tipo de plantio
abaixo é o seu próprio conjunto de scatter, e um conjunto só cresce nos seus alvos, as superfícies que você escolhe para ele.

## Monte um conjunto { #set-up-a-set }

Todo conjunto desta página começa do mesmo jeito.

1. Escolha a ferramenta Scatter

    Aperte **B**, ou clique no botão Scatter na barra de ferramentas. A aba Scatter abre à direita, ao lado do Inspetor.
    (Se você a fechar, **Painel scatter** na barra de opções da ferramenta a traz de volta.)

    ![A barra de ferramentas com o botão Ferramenta Scatter sob o mouse e a dica: arraste para pintar os modelos do conjunto de scatter ativo](../assets/sea-island/toolbar-scatter.png)

2. Comece de uma predefinição

    Clique em **Predefinição** no topo do painel Scatter e escolha a predefinição que o passo indica.

    ![O painel Scatter com o menu Predefinição aberto e floresta mista sob o mouse](../assets/sea-island/scatter-preset-menu.png)

3. Mire no terreno

    Clique no conta-gotas ao lado de **Alvos**, depois clique no terreno na vista 3D. Você deve vê-lo listado em
    Alvos, como `terrain #2` ou parecido.

    ![A linha Alvos do painel Scatter com o conta-gotas sob o mouse e a dica dele](../assets/sea-island/scatter-eyedropper.png)

Deixe **Seguir o cursor** desmarcado, já que ele pinta no que estiver sob o pincel e ignora os alvos. Deixe
**Manter espaçamento de outros conjuntos** marcado, para os conjuntos manterem distância uns dos outros e nada crescer atravessando um
tronco.

## Floresta { #woodland }

1. **Predefinição** > *floresta mista*, depois clique em **Renomear** no topo e chame de `woodland`. Mire no terreno.
2. Na seção Brush defina **Densidade** 0,16 e **Declive** 0 a 30, e marque **Altura**, 120 a 2000. A faixa de declive
    mantém as árvores longe dos penhascos, e a faixa de altura as mantém longe da praia.

    ![A parte de baixo do painel Scatter: terrain #2 como alvo, Seguir o cursor desmarcado, Densidade 0,16, Declive 0 a 30, Altura marcada de 120 a 2000, Manter espaçamento de outros conjuntos marcado e o botão Preencher alvos sob o mouse](../assets/sea-island/scatter-woodland-settings.png)

3. Clique em **Preencher alvos** na parte de baixo do painel para plantar o terreno todo de uma vez.

A densidade conta tentativas por 64 x 64 unidades, então 0,16 é mais ou menos uma árvore a cada 5 m.

![A floresta depois de Preencher alvos](../assets/sea-island/scatter-woodland.jpg)

Segure **Shift** e arraste ao longo de cada trilha com raio 260 para apagar as árvores dali. Apague uma clareira no
topo do morro principal (raio de uns 520) e um prado atrás da praia (uns 700).

![Apagando as árvores da trilha](../assets/sea-island/scatter-erase-paths.jpg)

## Árvores de destaque, arbustos e pedregulhos { #hero-trees-bushes-and-boulders }

Cada linha é um conjunto novo a partir da predefinição indicada, com o terreno como alvo, como na floresta. As árvores de destaque são poucas árvores grandes e
detalhadas, colocadas onde o jogador olha de perto.

| Conjunto | Predefinição | Como colocar | Raio | Densidade | Declive | Altura |
| --- | --- | --- | --- | --- | --- | --- |
| Árvores de destaque | floresta detalhada | Cliques únicos onde as trilhas saem da praia e em volta da clareira do topo do morro | 260 | 0,08 | 0 a 30 | desligada |
| Arbustos | arbustos | **Preencher alvos**, depois apague ao longo das trilhas com raio 190 | | 0,35 | 0 a 32 | 45 a 200 |
| Pedregulhos | pedregulhos | Um traço ao longo do topo dos penhascos | 1100 | 0,035 | 12 a 50 | 20 a 2000 |

![Árvores de destaque ao pé da trilha](../assets/sea-island/scatter-hero-trees.jpg)

![Arbustos numa faixa acima da areia](../assets/sea-island/scatter-bushes.jpg)

![Pedregulhos ao longo dos penhascos](../assets/sea-island/scatter-boulders.jpg)

## Grama { #grass }

1. Comece um conjunto da predefinição *grama*. A grama é folhagem, desenhada sem colisão nem sombras, então pode ser densa
   sem deixar o jogo lento.
2. Pinte os prados abertos com raio 650, densidade 2,5, declive 0 a 28, altura 45 a 2000. Apague-a das trilhas com
    raio 170.

    ![Grama de prado atrás da praia](../assets/sea-island/scatter-meadow.jpg)

3. Para grama nos pedregulhos, comece mais um conjunto de *grama*. Com o conta-gotas, clique num pedregulho em vez do terreno,
    assim o conjunto de pedregulhos vira o alvo. Você deve vê-lo em Alvos com `(set)` depois do nome. Depois
    **Preencher alvos** com densidade 6 e declive 0 a 45.

    ![A linha Alvos listando cliff boulders (set) como alvo](../assets/sea-island/scatter-target-boulders.png)

> [!NOTE]
> Preencher coloca grama nos pedregulhos como eles estão naquele momento, então termine os pedregulhos primeiro.

![Grama crescendo num pedregulho](../assets/sea-island/scatter-boulder-grass.jpg)

Próximo: [Ilha no mar 5: píer e Godot](sea-island-finish.md).
