---
source: 5b25efdda7a0b559f9cff81871758a2f55e33a9c
---

# Exportando para o Blender e outras ferramentas 3D { #exporting-to-blender-and-other-3d-tools }

Você pode salvar a aparência do seu mapa como um único arquivo de modelo 3D: brushes, malhas, terreno e modelos, com as texturas dentro. Útil para renderizar uma fase no Blender, montar uma cena de trailer, ou entregar um blockout a um artista que constrói as malhas finais em cima dele.

> [!WARNING]
> Só a geometria e os materiais são exportados. A lógica das entidades, as ligações de I/O, os triggers, os scripts e o comportamento de jogo ficam de fora, e o arquivo não pode ser aberto de novo no TrenchDot como um mapa. O `.tdm` continua sendo o mapa, siga editando ele.

## Passos { #steps }

1. Abra a exportação

    Vá em *Arquivo > Exportar > glTF Binary (.glb) para Blender e outras ferramentas 3D…*, ou digite "glTF" na paleta de comandos (Ctrl+Shift+P ou F1).

    ![O menu Arquivo com Exportar aberto e glTF Binary (.glb) para Blender e outras ferramentas 3D sob o mouse](../assets/exporting/menu-export.png)

2. Escolha o que entra

    A janela *Exportar para ferramentas 3D* abre. Marque o que você quer, veja [as opções](#options) abaixo. Os números ao lado de *Modelos de props* e *Instâncias de scatter* dizem quantos objetos cada um acrescenta. Você pode trocar o formato para OBJ no topo.

    ![A janela Exportar para ferramentas 3D com glTF Binary escolhido, Mapa inteiro, Modelos de props (5) marcado e Instâncias de scatter (4081) desmarcado](../assets/exporting/export-dialog.png)

3. Salve

    Clique em **Exportar…** e escolha onde o arquivo vai ficar. A exportação roda em segundo plano com uma janela de progresso, então você pode continuar editando, e **Cancelar** a interrompe sem escrever nada. Quando termina, a barra de status diz quantos objetos, materiais e triângulos saíram.

4. Abra no Blender

    Use *File > Import > glTF 2.0 (.glb/.gltf)* e escolha o arquivo. Os padrões estão certos, o importador deixa o modelo com Z para cima por conta própria.

O arquivo guarda o mapa como ele estava quando você clicou em **Exportar…**. As alterações não salvas passam antes pelo salvamento automático, a menos que ele esteja desligado em *Arquivo > Preferências*. Se a barra de status nomear um material para o qual o projeto não tem imagem, esse sai sem textura.

## Opções { #options }

| Opção | O que faz |
| --- | --- |
| Mapa inteiro, Só a seleção | Tudo, ou só os objetos selecionados, ainda dentro das suas camadas e grupos |
| Camadas e objetos ocultos | Também exporta o que está oculto nas vistas. Desligado, você leva o que vê |
| Modelos de props | Os modelos que os props e outras entidades ponto mostram, no lugar, rotação e escala deles |
| Instâncias de scatter | Cada árvore, pedra e tufo dos [conjuntos de scatter](scatter.md) como um objeto próprio. Desligado por padrão, já que milhares deles dão um arquivo grande e lento para abrir |
| Entidades pontuais como marcadores vazios | Um objeto vazio com o nome da entidade onde fica cada entidade ponto sem modelo, útil para posicionar luzes ou câmeras no Blender |
| Mesclar brushes sem nome | Os brushes que você não renomeou se juntam em um objeto por camada, grupo ou entidade, o que mantém curto o outliner do Blender. Os brushes com nome continuam separados |

## O que é exportado { #what-is-exported }

| No mapa | No arquivo |
| --- | --- |
| Camadas e grupos | Objetos pai vazios com os mesmos nomes, então a organização do Outliner passa junto |
| Brushes | Um objeto cada, nomeado como no Outliner, `brush12` ou o nome que você deu |
| Entidades de brush, como portas e elevadores | Um objeto pai com o nome da entidade, que guarda os brushes dela, onde eles estão no editor |
| Malhas, decals | Um objeto cada. Os decals ganham materiais mesclados e de face dupla |
| Terreno | Um objeto. Cada triângulo recebe a textura da sua camada pintada mais forte, então as misturas pintadas ficam em degraus |
| Instâncias de prefab | O conteúdo do prefab movido para o lugar, sob um objeto com o nome da instância |
| Materiais | Um por material com sua textura de albedo, mais os mapas de normal, rugosidade, metálico, oclusão ambiente e emissão que ele tiver. Os materiais recortados e translúcidos mantêm o alpha, e o pixel art continua nítido. As cores de desenvolvimento embutidas saem como suas texturas de grade |

O arquivo deixa de fora o que a construção no Godot não desenha: faces com texturas de ferramenta (clip, skip, origin, sky, trigger, nodraw, hint), faces escondidas contra outro brush, triggers e outras entidades de volume, e camadas marcadas com *Alternar omitir na exportação*. Luzes, sons, pontos de spawn, o céu e a névoa também não são exportados.

Só os objetos que vão para o arquivo escondem as faces uns dos outros. Então um pilar exportado com *Só a seleção* mantém a base que apoia no chão, e uma camada marcada para omitir na exportação não deixa buracos nas vizinhas.

## Unidades e eixos { #units-and-axes }

Um metro no Godot é um metro no Blender. As unidades do mapa são divididas pelas unidades por metro do projeto, 32 a menos que as configurações do mapa digam outra coisa. O arquivo tem Y para cima como o Godot, que é o que o glTF espera. Cada brush, malha e terreno tem a origem no centro dos seus limites, e os modelos mantêm a origem do arquivo de modelo.

## Quando usar OBJ { #when-to-use-obj }

*Arquivo > Exportar > Arquivo Wavefront OBJ (.obj)…* serve para ferramentas que não leem glTF. Ele escreve três coisas lado a lado, por exemplo `level.obj`, `level.mtl` com os materiais, e uma pasta `level_textures` com cópias das texturas. Mantenha-os juntos quando for movê-los. No Blender, use *File > Import > Wavefront (.obj)*.

O OBJ não tem objetos pai nem jeito de compartilhar uma malha entre cópias, então cada objeto é escrito achatado no espaço do mundo e cada instância de scatter é uma cópia completa. Com as instâncias de scatter marcadas, a janela avisa quando elas somam um milhão de triângulos e diz o tamanho que o arquivo vai ter. Uma exportação OBJ recusa mais de 10 milhões de triângulos, um arquivo de mais de um gigabyte. Os materiais mantêm as texturas de albedo, normal, rugosidade e emissão. Prefira o glTF sempre que a outra ferramenta o ler.

Um arquivo `.glb` guarda no máximo 4 GB. O glTF escreve cada modelo uma vez, não importa quantas instâncias de scatter o mostrem, então só um mapa enorme chega lá, e a exportação então avisa em vez de escrever um arquivo quebrado.

## Linha de comando { #command-line }

`trenchdot --export-glb <map> <out.glb>` e `--export-obj <map> <out.obj>` exportam com as opções padrão sem abrir uma janela. O projeto Godot do mapa é encontrado a partir da pasta dele. Veja [Linha de comando](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

`map_file {op: export_glb, path}` escreve um `.glb` e `map_file {op: export_obj, path}` um `.obj` com seu `.mtl` e a pasta de texturas, a partir do mapa aberto como ele está, salvo ou não. As opções são flags: `selection_only`, `hidden`, `scatter`, `models` (verdadeiro a menos que definido), `markers` e `merge_brushes`. O resultado lista `objects`, `meshes`, `materials`, `images`, `triangles`, `bytes` e `missing_textures`. O `run_action` recusa `export_glb` e `export_obj`, já que as entradas do menu abrem uma caixa de diálogo de arquivo.

</div>
