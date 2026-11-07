---
source: 3df4b9a4462dd78307ebb473881223aa02ea1321
---

# Iluminação noturna { #night-lighting }

Para um mapa noturno você baixa a luz global, para que as lâmpadas que você posiciona façam o trabalho.

## Escureça o mapa { #darken-the-map }

Aperte Esc, ou clique num espaço vazio em uma vista, para que nada fique selecionado. O Inspetor à direita então mostra
*Mapa (worldspawn)*, as configurações do mapa inteiro.

Defina as keys abaixo. Se uma key ainda não está na lista, digite no campo *chave* lá embaixo, o valor dela ao lado, e
clique em **Adicionar**.

| Key | Padrão | O que fazer para a noite |
| --- | --- | --- |
| `sun_energy`, `sun_color` | 1.1, `255 245 224` | Baixe a energia e escolha um azul pálido, e o sol vira lua |
| `ambient_energy`, `sky_energy` | 1.0, 1.0 | Baixe as duas, para que as áreas sem luz e o céu escureçam |
| `glow_intensity` | 0 | Acima de 0, as lâmpadas e os [materiais que brilham](materials.md#glowing-materials) ganham bloom no Godot |
| `ssr` | 0 | 1 liga os reflexos no espaço da tela, para que ruas molhadas e pisos brilhantes espelhem as lâmpadas |

É isto que o showcase do distrito noturno usa:

![O Inspetor mostrando Mapa (worldspawn) do distrito noturno: ambient_energy 0.8, glow_intensity 0.9, sky_energy 0.6, um sun_color azul pálido e sun_energy 0.16](../assets/night-lighting/worldspawn-night.png)

## Confira na Prévia iluminada { #check-it-in-lit-preview }

Aperte F3, ou clique em Prévia iluminada na barra de ferramentas, para ver o mapa com suas luzes, céu e névoa.

![A barra de ferramentas com o botão Prévia iluminada sob o mouse e a dica dele, Sombreamento: Prévia iluminada (F3)](../assets/night-lighting/toolbar-lit-preview.png)

Você deve ver as lâmpadas, as janelas acesas e os letreiros que brilham se destacando contra um céu escuro:

![A rua do distrito noturno na Prévia iluminada: postes de luz, janelas acesas e o letreiro brilhante da Corner Shop sob um céu escuro](../assets/night-lighting/street-lit-preview.jpg)

A Prévia iluminada mostra todas as keys acima, exceto o brilho e os reflexos, então confira esses no Godot. Para ter a luz
rebatida das lâmpadas sem pagar por ela a cada quadro, [pré-calcule a iluminação](../editor/light-baking.md).

## Luminárias { #lamp-fixtures }

Uma lâmpada é uma entidade `light` ou `light_spot` mais a geometria da luminária com o bulbo que brilha. Dê um targetname a essa geometria e
ponha-o na key `fixture` da luz (um `*` no fim casa com um prefixo).

A geometria então segue a luz sem nenhuma ligação: aparece enquanto a luz está ligada e some enquanto está desligada. Defina
`fixture_off` como `dark` para mantê-la visível com o brilho desligado.

![O Inspetor de uma luz com targetname alley_lamp, fixture alley_bulb e fixture_off definido como dark (emissão desligada)](../assets/night-lighting/light-fixture-keys.png)

Veja [Props, luzes e efeitos](../gameplay/entities/effects.md) para as outras keys de luz.

<div class="td-mcp" markdown>

## MCP { #mcp }

Os mapas de showcase são construídos por [scripts MCP](../mcp-scripts.md) que controlam o editor pelo
[servidor MCP](../mcp.md). `examples/mcp/night_district.json` e `examples/mcp/withered_city.json` são mapas noturnos completos
construídos assim.

</div>
