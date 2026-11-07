---
source: 3cc68efb1c3e922c9ae3811f442a5d5e642414b6
---

# Trim { #trim }

*Brush > Adicionar trim…* coloca tiras de brush ao longo das arestas das faces, saindo delas. Na base das paredes elas são rodapés, no alto uma moldura de teto, em volta de um piso uma borda, em volta de uma piscina o seu acabamento. Aqui está uma sala simples antes e depois de um clique:

![Uma sala rebocada com uma porta e as paredes selecionadas, ainda sem trim](../assets/trim/room-before.png)

![A mesma sala com rodapés escuros na base das paredes, subindo até a porta](../assets/trim/room-after.png)

1. (Opcional) Escolha o material do trim

    Aperte Escape para que nada fique selecionado, depois clique no material no painel Materiais. Ele vira o material atual sem alterar o mapa.

2. Selecione as paredes

    Toda face vertical dos brushes selecionados recebe trim. Para faces isoladas, use Shift+clique nelas, Ctrl+Shift+clique adiciona ou remove mais.

3. Abra a janela

    Escolha *Brush > Adicionar trim…*. Ela começa com o material atual, **Usar atual** volta a ele depois. No pé ela diz quantas faces encontrou.

4. Defina a tira

    Escolha quais **Arestas** recebem: base, topo, laterais ou todas. **Altura** é quanto a tira corre sobre a face, **Profundidade** quanto ela sai para fora, e **Recuo** deixa um vão entre a aresta e a tira.

    ![A janela Adicionar trim com Arestas em base, Altura 6 u, Profundidade 2 u, Recuo 0 u, showcase/planks_dark como material, Só onde há um piso sob a aresta marcado, Colisão desligada e 20 faces selecionadas](../assets/trim/add-trim-dialog.png)

5. Clique em **Adicionar trim**

    A barra de status diz quantas tiras ela fez.

As tiras caem num `func_detail_illusionary` enquanto **Colisão** está desligada, o padrão, para que os jogadores não esbarrem nelas. Marque a opção, ou use um jogo sem essa entidade, e elas caem num grupo simples.

Onde duas faces com trim se encontram, uma tira vai até o canto e a outra para nele, então os cantos fecham sem duas tiras se sobrepondo. Em volta da ponta de uma parede solta ou de uma porta a tira dá a volta. Uma tira que acabaria dentro de outro brush, como nas pontas de uma verga entre dois pedaços de parede, é deixada de fora.

**Só onde há um piso sob a aresta** mantém o trim da base nas arestas que apoiam em algo, então a parte de baixo de uma verga sobre uma porta não ganha rodapé.

> [!TIP]
> Trim são brushes comuns. Troque a textura, apague ou remodele tiras isoladas depois, como qualquer outro brush.

<div class="td-mcp" markdown>

## MCP { #mcp }

A ferramenta `trim` recebe `faces` como `[[brush, face], ...]`, ou `ids` de brushes, ou a seleção, mais `edges`, `height`, `depth`, `inset`, `material`, `on_floor` e `collision`. Ela devolve o novo grupo ou entidade e quantas tiras fez.

</div>
