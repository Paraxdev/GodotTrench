---
source: 7bb83afeb1e5ebd8423b507ac79851c690337899
---

# Preenchimento aleatório { #random-fill }

*Brush > Preenchimento aleatório…* cobre uma grade de células com peças da [biblioteca de prefabs](prefab-library.md), escolhendo a peça de cada célula pelo peso. Monte um kit pequeno de peças, como uma sala de escritório, um corredor, uma grade de pilares e um piso aberto, e o preenchimento monta uma planta baixa inteira com elas. Cada semente dá uma planta diferente, e a mesma semente sempre dá a mesma.

1. Monte suas peças

    Monte cada peça com uma célula de largura (128 unidades por padrão), centralizada na origem. Mantenha as paredes longe da borda da célula, para que peças vizinhas sempre se conectem.

    Depois adicione-a à biblioteca: selecione-a, abra **Adicionar a seleção** no topo da aba Prefabs, marque **Manter posição**, dê um nome e clique em **Adicionar** (veja [Suas próprias peças](prefab-library.md#your-own-pieces)).

    ![A aba Prefabs com Adicionar a seleção aberto e Manter posição marcado, a dica dizendo que mantém a origem do mapa como origem da peça, como um kit do Preenchimento aleatório](../assets/random-fill/prefabs-keep-position.png)

2. Abra a janela

    Vá em *Brush > Preenchimento aleatório…*.

    ![O menu Brush com Preenchimento aleatório… sob o mouse](../assets/random-fill/menu-random-fill.png)

3. Adicione as peças

    Escolha cada peça em **Adicionar peça…** e dê a ela um **peso**: uma peça com peso 6 preenche cerca do dobro de células de uma com peso 3. **girar** deixa uma peça girar por um múltiplo aleatório de 90 graus.

    ![A janela Preenchimento aleatório com três peças, seus pesos, peso vazio 4, uma grade de 12 por 8 e sua prévia colorida](../assets/random-fill/random-fill-dialog.png)

4. (Opcional) Deixe algumas células abertas

    **peso vazio** funciona como uma peça que não coloca nada.

5. Posicione a grade

    Selecione o piso e clique em **Ajustar à seleção**, ou defina você mesmo o tamanho da **Célula**, o número de **Células** e o **Canto**.

6. (Opcional) Agrupe as peças

    **Agrupamento** reúne cada peça em manchas de cerca de **Tamanho da mancha** células, para uma ala de escritórios ficar ao lado de um salão de pilares em vez de tudo ficar misturado por igual.

7. Preencha

    A grade colorida mostra o layout em prévia, uma cor por peça. Clique em **Preencher**, e **Sortear de novo** troca o último preenchimento pela próxima semente.

    ![Um piso preenchido com pilares quadrados e redondos e pilhas de caixotes, com algumas células deixadas vazias](../assets/random-fill/random-fill-result.jpg)

O preenchimento é um grupo de cópias simples, então ele é pré-calculado e construído como qualquer coisa que você posiciona à mão, e você pode editar qualquer célula depois. Os nomes de entidade de cada cópia recebem um prefixo próprio, `rf<seed>_<n>-`, então uma peça com entidades ligadas, como uma luminária que pisca num temporizador, funciona por conta própria em cada célula.

> [!TIP]
> Preencha tipos diferentes de detalhe em passadas separadas sobre a mesma grade, por exemplo um preenchimento para as paredes e outro para as luzes do teto, com seus próprios pesos, como algumas luzes queimadas ou piscando entre as que funcionam.

<div class="td-mcp" markdown>

## MCP { #mcp }

A ferramenta `random_fill` recebe `pieces` como `[{name, weight, rotate}]`, `empty`, `min`, `cell`, `cells`, `seed`, `cluster` e `cluster_size`. `preview: true` só devolve o layout como um esboço, `replace` remove um preenchimento anterior primeiro.

</div>
