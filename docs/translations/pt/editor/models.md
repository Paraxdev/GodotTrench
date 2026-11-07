---
source: d2dc61c8ee306263eda3f36520611e0790a27732
---

# Modelos { #models }

Modelos são arquivos 3D feitos em outro programa, como o Blender ou o Blockbench. O TrenchDot lê arquivos glTF, OBJ, Blockbench, STL, MD2 e MD3.

Você os encontra na aba **Modelos**, embaixo, ao lado de Materiais e Prefabs. Ela lista tudo que está em `res://models`, mais os modelos de natureza em `res://trenchdot/nature` depois que você os instalou.

![A aba Modelos: campo de busca, filtros de pasta e de origem, Ordenar, Slugs, o controle de tamanho das miniaturas, Ajustar, o multiplicador de tamanho, o botão de nova varredura e Importar…, acima de uma grade de miniaturas de modelos de natureza](../assets/models/models-panel.png)

Um projeto novo começa sem modelos. *Godot > Adicionar conteúdo ao projeto…* adiciona os modelos de natureza, e a opção da demo também traz os props do Poly Haven em `res://models/polyhaven`. Ou copie seus próprios arquivos para `res://models` e aperte o botão ⟳ para varrer de novo.

## Posicionar um modelo { #placing-a-model }

Um modelo entra no mapa de duas maneiras. Escolha conforme você queira mudar a forma dele no editor ou não.

| Posicionado como | Guardado no mapa | Use para |
| --- | --- | --- |
| Malha editável | Uma cópia da geometria, que não acompanha mais o arquivo | Formas que você quer editar com as [ferramentas de malha](meshes.md) |
| Entidade `prop_model` | Só um caminho `res://`, o Godot carrega o arquivo | Props que você mantém como estão. Toda cópia acompanha o arquivo quando você o muda |

### Como malha editável { #as-an-editable-mesh }

1. (Opcional) Defina o tamanho

    Ajustar, na barra de ferramentas Modelos, vem marcado por padrão. Ele escala para 2 metros um modelo cujo lado mais longo tem menos de um quarto de metro ou mais de 256 metros, para que arquivos minúsculos ou enormes ainda entrem utilizáveis. O campo × ao lado multiplica o tamanho por cima disso. Os dois valem na hora de posicionar, então defina-os antes de arrastar.

2. Arraste para uma vista

    Arraste uma miniatura da aba Modelos para qualquer vista. Ela cai na superfície sob o ponteiro.

    ![Um barril de vinho arrastado da aba Modelos para a vista 3D, onde cai no chão como uma malha editável](../assets/models/drag-model-into-view.gif)

Para um arquivo em qualquer lugar do disco, use **Importar…** no fim da barra de ferramentas Modelos.

### Como prop { #as-a-prop }

1. Escolha o arquivo

    Vá em *Arquivo > Importar > Prop de modelo (.bbmodel, .glb)…* e escolha o modelo.

    ![O menu Arquivo com Importar aberto e Prop de modelo (.bbmodel, .glb)… sob o mouse](../assets/models/file-import-model-prop.png)

2. Confira o Inspetor

    Ele cai no cursor 3D como uma entidade `prop_model`, e a key `model` guarda o caminho `res://`.

    ![O Inspetor de uma entidade prop_model com a key model apontando para um caminho res://models/polyhaven](../assets/models/inspector-prop-model.png)

Um prop precisa apontar para um arquivo que o Godot consiga carregar. Escolha um fora do projeto e o editor se oferece para copiá-lo antes para `res://models`.

Os modelos do Blockbench têm mais duas entradas, *Arquivo > Importar > Modelo do Blockbench como malha…* e *Modelo do Blockbench como brushes…*. Como brushes, cada cubo vira um brush, agrupado sob o nome do arquivo.

## Texturas de uma malha posicionada { #textures-of-a-placed-mesh }

Uma malha editável mantém o visual do modelo. As texturas dela são copiadas para `models/<name>` na pasta de texturas do projeto, onde `<name>` é o caminho do modelo na aba Modelos, ou o nome do arquivo para um modelo de outro lugar. Selecione a malha e o Inspetor lista os materiais com que ela é desenhada:

![O Inspetor de uma malha posicionada: 5966 vértices e o material models/polyhaven/containers/wine_barrel_01/tex0](../assets/models/inspector-mesh-materials.png)

Onde a imagem simples não seria desenhada como o modelo, um material do Godot também vai para a pasta de materiais do projeto. Assim folhas recortadas, pixel art e partes brilhantes (a cor e o mapa de emissão de um modelo glTF incluídos) aparecem no Godot como aparecem aqui. Para trocar os materiais, clique em um na aba Materiais com a malha selecionada, *Editar > Desfazer* os traz de volta.

Posicionar nunca sobrescreve um arquivo. Posicione o mesmo modelo de novo e ele reaproveita as texturas, e mantém um material que você ajustou no Godot. Se as texturas de outro modelo já estão nessa pasta, como as de `rock.bbmodel` ao lado de `rock.glb`, as novas vão para uma pasta numerada como `models/rock_2`. Desfazer deixa os arquivos onde estão. Se as texturas não puderem ser gravadas, digamos num projeto somente leitura, a malha entra sem textura e a barra de status diz o porquê.

Texturas contam como iguais pelos pixels, então uma que outro programa só salvou de novo ainda é reaproveitada. Pintou por cima de uma? As malhas posicionadas antes mantêm a sua versão, e o próximo posicionamento recebe cópias novas numa pasta numerada, com um material novo. Para dar suas alterações aos posicionamentos seguintes, aplique seu material a eles pela aba Materiais.

> [!NOTE]
> O FuncGodot só monta o material de uma face a partir do `default_material` do projeto quando não há um arquivo de material com o mesmo nome. As faces de um modelo posicionado que ganhou um arquivo de material são desenhadas com esse arquivo, então um shader próprio definido como `default_material` não chega a elas. Edite o arquivo no Godot para mudá-las. Um projeto cuja extensão de material não é `tres` não recebe arquivos de material, e o material padrão dele vale.

## Mais sobre uma miniatura { #more-on-a-thumbnail }

Clique com o botão direito em uma miniatura para posicioná-la no cursor, copiar o caminho dela ou achar o arquivo no disco. *Abrir página do pacote* aparece quando o pacote dela tem uma url.

![O menu do botão direito na miniatura wine_barrel_01 com Posicionar no cursor sob o mouse, depois Copiar caminho, Mostrar no gerenciador de arquivos e Abrir página do pacote](../assets/models/thumbnail-menu.png)

## Créditos do pacote { #pack-credits }

Pacotes gratuitos de modelos costumam pedir crédito. Ponha um `pack.json` na pasta de um modelo, ou em qualquer pasta acima dela (vale o mais próximo), e o painel sabe de que pacote o modelo veio, quem o fez e sob qual licença. Passe o mouse sobre uma miniatura para ver:

![A dica da miniatura wine_barrel_01: o caminho dela, Pacote: Poly Haven, o autor, Licença: CC0, a url do pacote e o crédito do modelo](../assets/models/thumbnail-credit-tooltip.png)

```json
{
  "name": "Nature Kit",
  "short": "Nature",
  "author": "Some Artist",
  "license": "CC0",
  "url": "https://example.com/nature-kit",
  "credits": [{ "asset": "pine", "author": "Other Artist" }]
}
```

`short` é o rótulo em cada miniatura enquanto Slugs está marcado, e recorre a `name`. `credits` sobrepõe o autor, a licença ou a url de modelos isolados, combinados pelo nome do arquivo sem extensão. Pastas sem `pack.json` são listadas sob o nome da pasta de nível mais alto delas.
