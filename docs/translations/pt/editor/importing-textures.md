---
source: 078f68d4f428db30dc2dfcb8561a8991ada14154
---

# Convertendo texturas { #converting-textures }

O Godot não consegue carregar arquivos `.vtf`, `.wad` ou `.wal`, então o editor os transforma em PNGs, mais um `.tres` onde uma imagem simples não consegue levar as configurações. Cada um vai parar na pasta de texturas do projeto, com o nome do material da face, de modo que as faces importadas o encontram na hora.

Importar um mapa oferece isso para as texturas que ele usa, veja [Importando mapas do Hammer e do TrenchBroom](importing.md). Para converter uma pasta inteira, abra primeiro o projeto do Godot, depois:

1. Abra o conversor

    Vá em *Arquivo > Importar > Converter texturas (VTF/VMT, WAD, WAL)…*.

    ![O menu Arquivo com Importar aberto e Converter texturas (VTF/VMT, WAD, WAL)… sob o mouse](../assets/importing-textures/file-import-convert-textures.png)

2. Escolha a pasta

    Escolha a pasta com os arquivos `.vmt` e `.vtf`, `.wad` ou `.wal`. Se o mapa aberto não tiver algumas delas, o editor pergunta se deve converter só **As do mapa** ou **Tudo**.

    A barra de status deve então dizer quantas foram convertidas, como "Texturas: 12 converted", e elas aparecem na aba Materiais com os nomes dos seus materiais.

    ![A aba Materiais buscando castle, mostrando brick_wall e stone_step de um mapa importado, com a dica castle/brick_wall, 1024 x 1024 px](../assets/importing-textures/converted-in-materials.png)

## Onde elas são encontradas { #where-they-are-found }

O editor procura nos lugares de costume onde cada jogo guarda suas texturas, então um mapa copiado junto com as pastas do jogo as encontra sem nenhuma configuração.

| Origem | Onde procura | Nome dado |
| --- | --- | --- |
| `.vmt` e `.vtf` da Valve | uma pasta `materials` ao lado do mapa ou até quatro pastas acima | o caminho abaixo de `materials`, como `brick/brickwall001a` |
| `.wad` do Quake e do Half-Life | a chave `wad` do worldspawn, depois o mesmo nome de arquivo em pastas próximas | o nome da textura, como `{grate` |
| `.wal` do Quake 2 | uma pasta `textures` por perto | o caminho abaixo de `textures`, como `e1u1/floor1_1` |
| `.png`, `.jpg`, `.tga` soltos | uma pasta `textures` por perto, como os projetos do Qodot e do FuncGodot as trazem | o caminho abaixo de `textures`, copiado como está |

## Materiais da Valve { #valve-materials }

Um arquivo `.vmt` da Valve descreve um material como uma lista de parâmetros. Estes passam para o material do Godot, o resto fica de fora:

| Material da Valve | Material do Godot |
| --- | --- |
| `$basetexture` | albedo, com o alpha descartado, a menos que o material seja translúcido |
| `$bumpmap`, ou `$normalmap` na água | mapa normal, com o verde invertido do DirectX para o Godot |
| `$translucent`, `$additive` | mistura alpha para `$translucent`, mistura aditiva para `$additive` |
| `$alphatest`, `$alphatestreference` | alpha scissor nesse limite, 0,5 por padrão |
| `$selfillum`, `$selfillummask` | uma textura de emissão brilhante, mascarada pelo alpha da textura base ou pela máscara |
| `$nocull`, `UnlitGeneric` | face dupla para `$nocull`, sem sombreamento para `UnlitGeneric` |
| `$color` | cor do albedo |
| `$surfaceprop` | `metadata/surfaceprop` no material |
| `patch` com `include` | um patch reaproveita outro material com algumas mudanças, então vira o material incluído com suas mudanças de `insert` e `replace` aplicadas |

## Texturas do Quake { #quake-textures }

As texturas que começam com `{` são recortadas com alpha scissor, os céus ficam sem sombreamento, e `*water` é salvo como `water`. As texturas animadas, incluindo VTFs com vários quadros, ficam com o primeiro quadro.

## Céus { #skies }

Converter as texturas de um mapa também define o céu dele, então o mapa importado ganha um céu parecido com o original. Isso é pulado quando o mapa já tem `sky_top_color`.

Um mapa do Source com seu skybox `skyname` em `materials/skybox` ganha um panorama, `skybox/<skyname>_panorama.png` na pasta de texturas, definido como `sky_panorama`. Os outros mapas ganham `sky_top_color` e `sky_horizon_color` a partir da textura em `sky_source`. Quando a cena não tem um WorldEnvironment próprio (o nó do Godot que guarda o céu, a névoa e a luz ambiente de uma cena), o `TrenchDotEnvironment` constrói um a partir dessas chaves.

## Formatos de VTF { #vtf-formats }

O conversor lê as versões 7.0 a 7.5 do VTF nestes formatos de pixel: DXT1 (com alpha de um bit), DXT3, DXT5, as ordens RGB e RGBA de 8 bits, I8, IA88, A8, RGB565, BGR565, BGRA4444, BGRA5551, BGRX5551, UV88, UVWQ8888, UVLX8888 e os formatos RGBA de 16 bits. Os outros formatos, e o VTF 7.6, são recusados com o nome do formato, para você saber qual arquivo converter à mão.

<div class="td-mcp" markdown>

## MCP { #mcp }

Pelo [servidor MCP](../mcp.md), converta uma pasta de texturas com `map_file {op: convert_textures, path}`. Adicione `only_missing` para converter só o que o mapa aberto não tem, e `overwrite` para substituir arquivos que já estão no projeto.

</div>
