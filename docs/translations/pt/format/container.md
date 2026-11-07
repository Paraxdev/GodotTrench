---
source: 8142a4423329f307647582ed665e04b916677fd2
---

# Estrutura do contêiner { #container-layout }

Um `.tdm` binário é um cabeçalho de arquivo de 12 bytes, seguido de chunks colados um no outro até o fim do arquivo. Todo número é little endian e de largura fixa.

| Deslocamento | Tamanho | Significado |
| --- | --- | --- |
| 0 | 8 | Magic `89 47 54 4D 0D 0A 1A 0A`, ou seja, `\x89GTM\r\n\x1a\n` |
| 8 | 4 | Versão do contêiner, u32, atualmente 1 |

Um leitor recusa uma versão de contêiner mais nova que a sua.

O `\r\n` no magic pega um arquivo que passou por uma conversão de fim de linha, no git por exemplo. Um leitor relata esse arquivo como corrompido em vez de lê-lo.

## Chunks { #chunks }

Cada chunk é um cabeçalho de 24 bytes e depois o seu payload. O marcador de sincronia e a verificação de cabeçalho distinguem um cabeçalho de chunk de verdade de bytes aleatórios, é assim que um leitor acha o próximo chunk bom num [arquivo danificado](recovery.md).

| Deslocamento | Tamanho | Significado |
| --- | --- | --- |
| 0 | 4 | Marcador de sincronia `89 47 54 43`, ou seja, `\x89GTC` |
| 4 | 4 | Tag, quatro caracteres ASCII imprimíveis como `NODE` ou `END ` |
| 8 | 4 | Flags, u32. O bit 0 indica que o payload é um frame zstd |
| 12 | 4 | Tamanho bruto, u32, o tamanho do payload depois de descomprimido |
| 16 | 4 | Tamanho armazenado, u32, o número de bytes de payload que vêm depois do cabeçalho |
| 20 | 4 | Verificação do cabeçalho, u32: a tag lida como u32, XOR flags, XOR tamanho bruto, XOR tamanho armazenado, XOR `0x47544D43` |

Um payload comprimido é um frame zstd com checksum de conteúdo. Um tamanho bruto acima de 1 GiB significa que o cabeçalho está danificado, e um arquivo inteiro também tem [limites](#limits).

O editor grava os chunks nesta ordem:

1. Um chunk `HEAD`.
2. Os chunks `NODE`.
3. Os chunks desconhecidos que ele manteve do arquivo que carregou.
4. Um chunk `LMAP`, quando o mapa tem iluminação pré-calculada.
5. O chunk `END`, sempre por último.

### HEAD { #head }

O [objeto de nível superior](README.md#top-level) sem `layers`.

### NODE { #node }

Um lote de nós inteiros:

| Chave | Tipo | Significado |
| --- | --- | --- |
| `parents` | array int64 | O id do pai de cada nó, 0 para uma layer |
| `nodes` | array de objetos de nó | Os nós, cada um sem `children` |

Os nós vêm na ordem da árvore, então um pai sempre vem antes dos filhos, no mesmo chunk ou num anterior. O editor começa um chunk novo quando um lote passa de 64 KiB antes da compressão, e nunca divide um nó entre dois chunks. Assim um chunk danificado só leva consigo os próprios nós.

Para reconstruir a árvore, anexe cada nó aos `children` do nó que o id de pai dele indica.

### END { #end }

O chunk `END` deixa um leitor conferir que nada está faltando:

| Chave | Tipo | Significado |
| --- | --- | --- |
| `version` | inteiro | A versão do mapa, para os nós ainda carregarem quando o `HEAD` está danificado |
| `nodes` | inteiro | Quantos nós os chunks `NODE` guardam |
| `chunks` | inteiro | Quantos chunks vêm antes do `END` |
| `content` | inteiro | Id de conteúdo, veja abaixo |

O id de conteúdo é um hash FNV-1a de 64 bits dos payloads brutos de `HEAD` e `NODE` na ordem do arquivo, cortado nos 52 bits baixos para sobreviver como número JSON. O Godot o guarda junto com a cena construída, assim uma [sessão ao vivo](../godot/live-link.md) consegue saber que um mapa não mudou e pular a reconstrução.

### LMAP { #lmap }

A [iluminação pré-calculada](../editor/light-baking.md), presente só quando o mapa tem um bake. É um único dicionário com 18 chaves, num chunk só dele, para o resto do mapa carregar rápido e um editor mais antigo mantê-lo intacto. Como é grande, o editor o comprime com um nível de zstd mais rápido que o dos outros chunks. Posições e distâncias estão em unidades do mapa.

| Chave | Tipo | Significado |
| --- | --- | --- |
| `version` | inteiro | Versão da estrutura, atualmente 1. Um leitor descarta um bake com uma versão mais nova |
| `width`, `height` | inteiro | Tamanho do lightmap em texels |
| `scene` | inteiro | Impressão digital da geometria, das luzes e do céu a partir dos quais o bake foi feito, para saber quando ele está desatualizado |
| `texel_size` | float | Unidades do mapa por texel |
| `light` | bytes | Luz linear, três half floats little endian (vermelho, verde, azul) por texel, linha por linha |
| `shadow` | bytes | Quanto do sol chega a cada texel, de 0 a 255 |
| `ao` | bytes | Oclusão ambiente por texel, 255 onde não há nada perto |
| `chart_keys` | array int64 | Um id de nó e um índice de face por chart, a face é 0 para um terreno |
| `chart_rows` | array float32 | Oito floats por chart, duas linhas `a b c d` que dão a coordenada de 0 a 1 do chart no lightmap como `a*x + b*y + c*z + d` |
| `nodes` | array int64 | Pares de um id de nó e a impressão digital da geometria dele quando foi calculado |
| `stale` | array int64 | Nós cuja geometria mudou desde então, ou que sumiram. Os leitores pulam os charts deles |
| `fallback` | array float32 | A luz média como vermelho, verde e azul, para faces sem chart |
| `probe_points` | array float32 | Posições das sondas de luz, três floats cada |
| `probe_sh` | array float32 | Nove coeficientes de harmônicos esféricos de vermelho, verde e azul por sonda, como o Godot os guarda |
| `probe_tetrahedra` | array int32 | Quatro índices de sonda por tetraedro |
| `probe_bsp_planes` | array float32 | Um plano por nó BSP, normal e distância |
| `probe_bsp_children` | array int32 | O filho acima e abaixo de cada plano: outro nó quando positivo, o tetraedro `t` como `-t - 1`, nada como o menor int32 |

As chaves de sonda seguem a estrutura do `LightmapGIData` do Godot, e um leitor deixa as sondas de fora quando os tamanhos não batem. Um chunk que falha ao carregar é descartado com um aviso para calcular de novo.

Num mapa JSON o mesmo dicionário fica sob uma chave `lightmap` de nível superior, com `light`, `shadow` e `ao` como texto base64.

### Outros chunks { #other-chunks }

Os leitores pulam um chunk com uma tag que não conhecem. O editor o mantém byte a byte e o grava de volta depois dos seus chunks `NODE` ao salvar.

Isso faz de um chunk novo o jeito de adicionar dados que os editores mais antigos levam adiante sem entender. Lembre que um editor mais antigo pode ter apagado nós a que o seu chunk se refere, então um tipo novo de chunk precisa continuar válido quando esses nós não existem mais.

## Codificação dos valores { #value-encoding }

Cada payload é um valor na serialização binária de Variant do Godot, o formato do `var_to_bytes`, usando só os tipos abaixo. Todo valor começa com um cabeçalho u32 cujo byte baixo é o tipo. O bit 16 do cabeçalho marca um inteiro ou float de 64 bits. O bit 16 em qualquer outro tipo, ou qualquer outro bit, torna o chunk ilegível. O mesmo vale para um valor dentro de mais de 128 arrays e dicionários aninhados.

| Tipo | Cabeçalho | Corpo |
| --- | --- | --- |
| null | 0 | Nada |
| bool | 1 | u32, 0 ou 1 |
| inteiro | 2 | i32, ou i64 com o bit 16 ligado |
| float | 3 | f32, ou f64 com o bit 16 ligado. O editor sempre grava f64 |
| string | 4 | Comprimento em bytes u32, depois os bytes UTF-8, depois zeros de preenchimento até um múltiplo de 4 |
| dicionário | 27 | Contagem u32, depois cada chave (uma string) seguida do seu valor |
| array | 28 | Contagem u32, depois os valores |
| bytes | 29 | Comprimento u32, depois os bytes, depois zeros de preenchimento até um múltiplo de 4 |
| array int32 | 30 | Contagem u32, depois um i32 cada |
| array int64 | 31 | Contagem u32, depois um i64 cada |
| array float32 | 32 | Contagem u32, depois um f32 cada |
| array float64 | 33 | Contagem u32, depois um f64 cada |

Os leitores ignoram o bit 31 da contagem de um dicionário ou array, que o Godot liga para contêineres compartilhados, mas não o da contagem de um array empacotado. Inteiros JSON são gravados como inteiros e floats JSON como floats, então converter para binário e voltar dá o mesmo JSON.

Para manter os arquivos pequenos, três tipos de valor ganham uma forma mais compacta. Um leitor precisa aceitar tanto a compacta quanto a simples:

| Valor JSON | Guardado como | Quando |
| --- | --- | --- |
| Array de 32 ou mais números | Um array int32 quando todos são inteiros que cabem. Um array float32 quando todos são floats que um float de 32 bits guarda com exatidão. Um array float64 para outros floats | Sempre |
| `heights`, `splat` e `holes` do terreno (texto base64) | bytes, os dados decodificados | Quando o base64 está na forma canônica |
| `instances` do scatter | Um array int32 de colunas | Quando todo valor é um float e um múltiplo inteiro do seu passo |

As colunas do scatter guardam primeiro todos os índices de item, depois todos os x, e assim por diante pelos oito campos de uma instância. Cada valor é guardado como uma contagem inteira do seu passo: 1 para o item, 100 por unidade para a posição, 10 por grau para os ângulos e 1000 para a escala. Dividir devolve o float exato, por exemplo `1234 / 100.0` é `12.34`.

> [!NOTE]
> O `tdm_file.gd` transforma as colunas do scatter de volta em arrays de instâncias, mas deixa os bytes do terreno como `PackedByteArray` e os arrays longos de números como arrays empacotados. O GDScript que lê um mapa precisa aceitar esses ao lado de arrays simples.

## Limites { #limits }

Uns poucos kilobytes de zstd podem se expandir para gigabytes, então um leitor define um orçamento por arquivo. Ele pega os chunks `HEAD`, `NODE`, `END` e `LMAP` na ordem do arquivo e não lê um que não caiba no que sobrou:

1. Antes de descompactar, o tamanho bruto do chunk sai de um orçamento de 1 GiB.
2. O chunk descompactado fica na memória enquanto seus valores são construídos, então o tamanho bruto dele e a memória dos seus valores precisam caber juntos no que sobrou de um orçamento de 1200 MiB, ao lado do que os nós de um chunk `NODE` ocupam depois que o mapa é construído a partir deles. Só a memória dos valores e dos nós sai dele. Um chunk que não cabe não leva nada, então os chunks depois dele ainda podem ser lidos.

Chunks com outras tags nunca são descompactados e não levam nada. A memória é contada assim, sem construir nenhum valor:

| Valor | Memória |
| --- | --- |
| Todo valor, incluindo chaves de dicionário e os itens de arrays empacotados | 80 bytes |
| Uma string, além disso | 6 bytes por byte, já que o JSON pode escrever um byte como `\u0000` |
| Bytes, além disso | `4 * ceil(n / 3) + n` para `n` bytes, o tamanho deles em base64 e a cópia para a qual um terreno ou bake os decodifica |
| Um nó de um chunk `NODE`, além dos seus valores | 5 KiB para uma entidade, 12 KiB para um conjunto de scatter, 1 KiB para qualquer outro nó |

Os custos dos nós são um limite superior do que os menores nós de cada tipo ocupam no editor depois que ele abre o mapa (um grupo ou brush chega a uns 650 bytes e é cobrado como 1 KiB), então um arquivo pequeno de muitos nós minúsculos não pode usar mais memória que o orçamento. Nós aninhados sob `children` dentro de um chunk `NODE` são descartados e o leitor relata quantos dos nós do chunk tinham algum, já que o editor nunca os grava ali. As instâncias de scatter só são contadas pelos seus valores, que chegam a mais que os cerca de 400 bytes que o editor guarda para uma instância construída, já que todas as instâncias de um conjunto compartilham o modelo na GPU.

O maior mapa de showcase ocupa 79 MiB, uma instância de scatter 640 a 720 bytes, e a luz, a sombra e a oclusão de um bake no maior tamanho de atlas 299 MiB. O editor se recusa a salvar um mapa além de qualquer um dos dois limites, então todo arquivo que ele grava é lido por inteiro.
