---
source: d3f0019b345d8a4b85cb01b22ccb9fd1cdee21d4
---

# O formato de mapa .tdm { #the-tdm-map-format }

Estas páginas são para você se está escrevendo uma ferramenta que lê ou grava arquivos `.tdm`, como um importador, um conversor ou uma verificação de CI. Você não precisa de nada disto para construir níveis.

Um arquivo `.tdm` guarda três coisas: as propriedades do worldspawn, um pouco de estado do editor e uma árvore de nós com o conteúdo de verdade.

O editor o salva como um contêiner binário de chunks comprimidos com zstd, então os arquivos ficam pequenos e um arquivo danificado ainda abre. Editores mais antigos salvavam um único documento JSON em UTF-8, e todo leitor carrega as duas formas. As duas guardam a mesma árvore, por isso estas páginas a mostram como o JSON que `trenchdot --dump` imprime.

| Página | Leia quando você |
| --- | --- |
| [Estrutura do contêiner](container.md) | Lê ou grava o arquivo binário. Cobre o cabeçalho do arquivo, os chunks e como cada valor é codificado em bytes |
| [Arquivos danificados](recovery.md) | Quer saber o que ainda carrega de um arquivo corrompido, e onde cada leitor relata o que se perdeu |
| [Nós](nodes.md) | Percorre a árvore. Explica a forma que todo nó compartilha, como os ids funcionam e os nós de layer, grupo, entidade e instância |
| [Geometria](geometry.md) | Lê ou monta brushes e malhas, incluindo suas faces, a projeção de textura e os displacements |
| [Terreno e scatter](terrain-scatter.md) | Trata terrenos de heightmap e conjuntos de modelos espalhados |
| [Estrutura do JSON](json.md) | Trabalha com a forma legível, converte entre as duas formas ou lê o que o editor coloca na área de transferência |

Quer ver código funcionando? A implementação de referência é o
[`format.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/format.rs), com o contêiner em
[`binary.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/binary.rs) e
[`variant.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/variant.rs). No lado do Godot, o
[`tdm_file.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_file.gd) lê o arquivo e o
[`tdm_parser.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_parser.gd) o constrói.

## Nível superior { #top-level }

| Chave | Tipo | Gravada | Significado |
| --- | --- | --- | --- |
| `format` | string | sempre | Sempre `"trenchdot-map"`. Distingue um mapa de outros JSONs |
| `version` | inteiro | sempre | Versão do mapa, atualmente 1 |
| `properties` | objeto de string para string | quando não está vazio | Chaves do worldspawn, ordenadas por nome |
| `editor` | objeto | quando nem tudo é padrão | Estado da vista do editor. Nunca é exportado para o Godot |
| `textures` | objeto | quando alguma textura tem configurações | [Configurações de textura](geometry.md#texture-settings) por nome de material |
| `layers` | array de nós | sempre | Os [nós de layer](nodes.md#layer) do mapa, em ordem. Todo o resto fica pendurado abaixo deles |
| `lightmap` | objeto | mapas JSON com um bake | A iluminação pré-calculada, que um mapa binário guarda no seu [chunk LMAP](container.md#lmap) |

Só os nós `layer` são lidos de `layers`, e um mapa sem nenhum ganha uma layer `Default` ao carregar.

Os valores do worldspawn são sempre strings, como num arquivo `.map`. Quando o Godot constrói o mapa, o `TrenchDotEnvironment` transforma essas chaves em céu, névoa e sol:

| Chaves | O que controlam |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | A luz direcional do sol: para onde aponta, sua cor e quão forte é |
| `ambient_color`, `ambient_energy` | A luz que chega às superfícies que o sol não alcança, para as sombras não ficarem totalmente pretas |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | O gradiente procedural do céu e seu brilho |
| `sky_panorama` | Uma imagem `res://` que substitui o céu procedural no Godot. A prévia do editor e o seu bake continuam usando as cores do céu |
| `fog_color`, `fog_density` | Névoa de distância |
| `glow_intensity`, `ssr` | Bloom nas superfícies brilhantes, e reflexos no espaço da tela quando `ssr` é `1` |

Se o seu jogo traz o próprio `WorldEnvironment`, defina `environment` como `sun_only` e só o sol é construído. `0` ou `none` não constrói nada, veja [ambiente e sol](../godot/building.md#environment-and-sun). O `sky_source` lembra qual textura as faces de céu de um mapa importado tinham.

O objeto `editor` é estado de vista que só importa dentro do editor:

| Chave | Tipo | Padrão | Significado |
| --- | --- | --- | --- |
| `cameras` | objeto, chaves `"1"` a `"9"` | vazio | Posições de câmera salvas, cada uma `{"position": [x, y, z], "yaw": r, "pitch": r}` com ângulos em radianos |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | nenhum | A caixa do cordon em unidades do mapa, usada para trabalhar numa parte de um mapa grande |
| `cordon_enabled` | bool | `false` | Esconde os objetos fora do cordon e os deixa de fora das exportações com cordon |
| `bake` | `{"quality", "texel_size", "softness", "backend"}` | nenhum | As opções com que o mapa foi [pré-calculado](../editor/light-baking.md) por último, para o próximo bake começar a partir delas |
| `graph_frames` | array | vazio | Frames do [grafo lógico](../gameplay/logic-graph.md#frames), cada um `{"title", "rect": [x, y, width, height], "color"}` em pontos do grafo. Um frame danificado é descartado ao carregar |

## Versões e compatibilidade { #versions-and-compatibility }

`format` e `version` descrevem os dados. A versão do contêiner no cabeçalho do arquivo é separada, e só muda quando a estrutura dos chunks muda.

Veja o que cada leitor faz com um arquivo que não entende por completo:

| Situação | Editor | Addon do Godot |
| --- | --- | --- |
| `format` diferente | Recusa o arquivo | Erro de build |
| `version` mais nova | Recusa o arquivo | Erro de build pedindo para atualizar o addon |
| `version` ausente | Lê como a versão mais antiga | Igual |
| Chave desconhecida | Ignora, e ela some depois do próximo salvamento | Ignora |
| Chunk desconhecido | Mantém e o grava de volta ao salvar | Pula |
| Índice de face fora do intervalo | Recusa o mapa, citando o nó | Pula esse brush |
| `heights` de displacement com o tamanho errado | Recusa o mapa, citando o nó | Constrói a face plana |

O editor descarta as chaves desconhecidas ao salvar. Então, se você adiciona dados que um editor mais antigo não pode perder, precisa de um aumento da versão do mapa, que os editores antigos se recusam a abrir, ou de um chunk próprio, que eles mantêm intacto. Veja [Outros chunks](container.md#other-chunks).

## Coordenadas e unidades { #coordinates-and-units }

As posições usam os eixos do Godot: Y para cima, X para a direita, Z em direção a quem olha. O build divide pelo `inverse_scale_factor` das configurações do mapa, 32 por padrão, então 32 unidades do mapa formam um metro.

| Grandeza | Unidade |
| --- | --- |
| Posições, tamanhos, alturas, espaçamento, tamanho de célula | Unidades do mapa |
| Ângulos de entidades, instâncias e scatter | Graus: pitch em torno de X, yaw em torno de Y, roll em torno de Z, aplicados na ordem YXZ como o `rotation_degrees` do Godot |
| Yaw e pitch do marcador de câmera | Radianos |
| Deslocamento de UV | Texels |
| Escala de UV | Unidades do mapa por texel |
| `uvs` da malha | Larguras de textura, então 1 é uma repetição completa da textura |
