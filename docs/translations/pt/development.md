---
source: d0d174df9061792086d11223ac4029e9d23f9d56
---

# Desenvolvimento { #development }

Esta página é para você se quiser compilar o editor a partir do código-fonte, mudar o addon ou escrever documentação.

## Compilando { #building }

Clone com os submódulos, o addon do Godot em `godot/addons/func_godot` é um deles:

```sh
git clone --recursive https://github.com/Paraxdev/TrenchDot.git
```

Já clonou sem isso? Rode `git submodule update --init`. O editor compila sem o addon, mas os testes e
o projeto Godot precisam dele.

```sh
cargo run -p td_editor --release            # start the editor
cargo run -p td_editor --release -- map.tdm # open a map
```

Você precisa de uma GPU com Vulkan, DX12 ou Metal e, no Linux, uma sessão X11 ou Wayland.

O Rust 1.95 é o mínimo, o egui 0.36 exige isso.
O [rust-toolchain.toml](https://github.com/Paraxdev/TrenchDot/blob/main/rust-toolchain.toml) fixa a 1.98.1, a versão
que o CI usa, então o rustup a instala na sua primeira compilação.

## Verificações { #checks }

Rode `./tools/fmt --install` uma vez por clone. Ele ativa um hook de pre-commit que formata o Rust em staging.

```sh
./tools/fmt --check
python tools/gen_licenses.py --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p td_editor --test e2e -- --ignored --test-threads=1
godot --headless --path godot --import                            # twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd
godot --headless --path godot --script res://tests/build_showcase.gd
```

O CI roda tudo isso, então rode antes de dar push.

Os testes e2e abrem o editor de verdade e o controlam com entrada simulada. Isso precisa de uma GPU e de uma sessão de desktop, então
o `cargo test` os pula a menos que você passe `--ignored`. O CI os roda no Linux com Vulkan por software. Para testar o
Live Link sem uma janela, veja [Protocolo do Live Link](godot/live-link.md#testing-without-a-window).

> [!NOTE]
> `godot --headless --script` não renderiza nada (capturas de tela e verificações de renderização saem pretas) e não
> roda os autoloads do projeto, só o próprio script. Escreva testes sobre dados e nós, não sobre pixels nem sobre o estado de autoloads.

Para medir o tempo de construção dos mapas, rode `godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`.
Não use `build_showcase.gd` para isso, ele sobrescreve as cenas de showcase já commitadas.

## Arquivos de mapa no git { #map-files-in-git }

Os mapas `.tdm` são binários, veja [o formato de mapa .tdm](format/README.md). O `.gitattributes` do repositório já os
marca. Para diffs legíveis, tudo o que você precisa é da linha de `git config` em
[Revisando alterações](editor/reviewing.md#maps-in-git), uma vez por clone.

## Conteúdo gerado { #generated-content }

Alguns arquivos são feitos por um script, não à mão. Quando você mudar o que um script lê, rode-o de novo.

| Comando | O que ele regenera |
| --- | --- |
| `cargo run -p td_formats --example demo_maps` | Os mapas de teste em `godot/tests/maps` e os pequenos mapas de demonstração em `godot/demo/maps` |
| `python tools/fetch_demo_textures.py` | As texturas fotográficas CC0 da demo, veja [tamanho de textura](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | Os props CC0 do Poly Haven em `godot/models/polyhaven`, com os mapas alfa de vidro e chamas restaurados. `--fix-alpha` só os restaura |
| `cargo run -p td_samples --bin showcase --release` | Os modelos Blockbench da demo |
| `cargo run -p td_editor --example export_addon_fgd` | Os arquivos FGD do núcleo do addon e o modelo do pacote de entidades de gameplay a partir de `builtin_entities.json` e `gameplay_entities.json`, escritos no submódulo do addon, e depois a cópia instalada do pacote no projeto de demonstração |
| `python tools/gen_licenses.py` | A tabela de crates Rust em [LICENSES.md](https://github.com/Paraxdev/TrenchDot/blob/main/LICENSES.md) |

## O addon do Godot { #the-godot-addon }

O addon tem o seu próprio repositório, [trenchdot_func](https://github.com/Paraxdev/trenchdot_func), obtido em
`godot/addons/func_godot`. Os testes dele ficam aqui em `godot/tests`, já que precisam do projeto de demonstração. Para mudar o addon:

1. Faça commit da mudança dentro de `godot/addons/func_godot` e dê push no trenchdot_func.
2. Coloque o novo commit do submódulo em staging aqui com `git add godot/addons/func_godot` e faça commit dele junto com as
   mudanças do TrenchDot que precisam dele.

Dê push no addon primeiro. Um commit do TrenchDot apontando para um commit do addon que o GitHub não tem falha ao fazer checkout no CI.
O `version` do `plugin.cfg` combina com a versão do workspace do [Cargo.toml](https://github.com/Paraxdev/TrenchDot/blob/main/Cargo.toml).
Aumente os dois juntos, o editor avisa quando o addon de um projeto não combina com ela.

## Traduções { #translations }

A interface do editor vem em inglês, alemão, francês, espanhol, português, russo, chinês simplificado e japonês, veja
[Idioma](editor/interface.md#language). O inglês é a fonte, e os outros sete ficam em
[crates/td_editor/locales](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales), um arquivo JSON
cada, embutido quando o editor é compilado. O README de lá tem o formato, as formas de plural e um glossário.

No código, o texto em inglês é a chave. Envolva um literal em `en!("...")` onde ele é guardado e em `tr(...)`,
`trf!("... {n} ...", n = count)` ou `tr_n!(count, "{n} brush", "{n} brushes")` onde ele é desenhado, e adicione a mesma chave
a todos os arquivos. Um texto sem entrada aparece em inglês. O código em `crates/td_editor/src/mcp`, as linhas de log e o conteúdo
dos mapas nunca passam por eles, as respostas e os erros do MCP continuam em inglês. Um `Msg` guarda o texto em inglês ao lado da tradução, então
a barra de status pode mostrar um enquanto o MCP lê o outro.

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor --lib i18n
```

Essa execução falha enquanto um arquivo não tiver uma chave que o código usa ou ainda guardar uma que ele não usa mais. Com `TD_L10N_TODO` definido para
uma pasta existente, ela também escreve um arquivo por idioma com as chaves que faltam. Outro teste varre o código-fonte atrás de texto de interface
que pula a busca, então um literal cru entregue a um rótulo o faz falhar. As traduções são escritas por IA e não foram conferidas, uma
correção de um falante nativo mexe só no arquivo daquele idioma. O editor iniciado com `--default-prefs`, como os testes e2e fazem,
fica sempre em inglês.

O manual também tem traduções, veja [Traduzindo o manual](#translating-the-manual). As páginas dele nomeiam itens de menu e
botões do jeito que o editor os mostra, então depois de mudar um valor em um catálogo rode
`python3 tools/translations.py check`. Ele lista as páginas que ainda usam a redação antiga.

## Documentação { #documentation }

A documentação são as páginas Markdown em `docs/`, e o [Zensical](https://zensical.org) as transforma no site.
O `zensical.toml` na raiz do repositório o configura, e o `docs/SUMMARY.md` é o sumário. Uma página que falta ali
ainda é construída, só não tem lugar na navegação. O `docs/requirements.txt` fixa o Zensical,
o `docs/stylesheets/td.css` guarda as cores, o `docs/javascripts/td.js` o interruptor de MCP e os cartões do glossário, e
o `docs/overrides` substitui alguns templates do tema.

Rode isto a partir da raiz do repositório:

```sh
python3 -m venv .venv                  # once
. .venv/bin/activate                   # .venv\Scripts\activate on Windows
pip install -r docs/requirements.txt   # once, and again when the pin changes
zensical serve                         # live preview of the English site on http://localhost:8000/TrenchDot/
python3 tools/translations.py check    # the translated pages against their English source
python3 tools/build_site.py            # the build CI runs, English at the root of _site and a folder per language
python3 tools/test_docs.py             # tests of the three tools and of the built site
python3 tools/check_site.py            # the link check CI runs on _site
```

O `build_site.py` roda `zensical build --clean --strict` uma vez para o inglês e uma para cada idioma, então falha em um
link para uma página ou um título que não existe em nenhum deles. Uma entrada errada no `SUMMARY.md`, uma imagem faltando ou uma página
fora da navegação passam por ele, o `tools/check_site.py` pega isso em todos os idiomas. Para construir só alguns
idiomas, nomeie-os: `python3 tools/build_site.py de ja`.

O id de um título é o texto dele em minúsculas, com os espaços virando hifens e a outra pontuação removida, então `## path_corner`
é `#path_corner`. Os links de ajuda do editor em
[help.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/help.rs) apontam para esses ids, e um teste
os confere, então só renomeie um título junto com os links dele.

Os destaques são alertas no estilo do GitHub: `> [!NOTE]`, `> [!TIP]` ou `> [!WARNING]` em uma linha própria, com o texto nas
linhas citadas abaixo. Assim eles aparecem também no GitHub. Tudo o que pertence a um passo numerado, um segundo
parágrafo, uma imagem ou um bloco de código, é indentado com 4 espaços, ou sai da lista.

As imagens de uma página ficam em `docs/assets/<page>/`.

Se um recém-chegado não conheceria um termo, dê a ele uma entrada em `GLOSSARY.md`: um título `## term` com uma definição de um
parágrafo. O primeiro uso dele em cada página passa então a linkar para ela e mostra a definição quando você passa o mouse. Código, links e
títulos ficam de fora.

O `.github/workflows/pages.yml` constrói o site para os pull requests que mexem em `docs/`, `zensical.toml`, nas ferramentas da documentação ou
nos catálogos do editor, e o publica no GitHub Pages quando chegam à `main`.

### Traduzindo o manual { #translating-the-manual }

O site também é construído em alemão, francês, espanhol, português, russo, chinês simplificado e japonês, os idiomas
do editor. O inglês é a fonte, e os outros idiomas se chamam como os códigos do editor: `de`, `fr`, `es`,
`pt`, `ru`, `zh-Hans` e `ja`. O site de um idioma fica em `/TrenchDot/<code>/`, e o editor abre o manual
ali quando roda nesse idioma. As traduções são escritas por IA e nenhum falante nativo as conferiu, então cada
site traduzido diz isso em um aviso que o leitor pode fechar. Uma correção de alguém fluente é bem-vinda.

**Onde ficam as coisas.** Uma página traduzida fica em `docs/translations/<code>/` com o nome da página em inglês,
`docs/translations/de/editor/brushes.md` para `docs/editor/brushes.md`. Ao lado das páginas, o `site.toml` guarda os textos
que não são páginas: o idioma do tema, a descrição do site, os nomes do botão de esquema de cores e do interruptor de MCP,
o aviso, a nota para páginas não traduzidas, os títulos dos alertas e uma tabela `[nav]`. A tabela `[nav]` mapeia cada
título em inglês do `SUMMARY.md` para a tradução dele, um título que falta ali aparece em inglês. A lista de idiomas é
`[[project.extra.alternate]]` no `zensical.toml`.

**A linha source.** Uma página traduzida começa com um front matter que nomeia a página em inglês que ela acompanha:

```yaml
---
source: 0123456789abcdef0123456789abcdef01234567
---
```

O `source` é o que `git hash-object docs/<page>` imprime para a página em inglês. Enquanto ele combina, a tradução está
atual e o build a usa. Quando a página em inglês muda ele deixa de combinar, a tradução fica desatualizada, e o
build mostra no lugar a página em inglês com uma nota curta depois do título. Então editar uma página em inglês nunca quebra o CI e
não exige edição de tradução, a página só fica em inglês nos outros sites até alguém atualizá-la. Uma página que falta cai
do mesmo jeito, então toda página existe em todo idioma e o botão de idioma sempre cai na mesma página.

**Comandos.** Todos rodam a partir da raiz do repositório.

| Comando | O que faz |
| --- | --- |
| `python3 tools/translations.py status` | Lista páginas desatualizadas, faltando e órfãs e títulos de navegação sem entrada. Nunca falha. Adicione `--lang de` para um idioma e `--diff` para ver o que mudou no inglês desde que cada página foi traduzida. |
| `python3 tools/translations.py prep --lang de editor/brushes.md` | Imprime o que um tradutor precisa para uma página: o front matter, os ids dos títulos, os rótulos com a tradução deles, as palavras do glossário e o texto em inglês. |
| `python3 tools/translations.py check [--lang de] [page ...]` | Falha em uma tradução atual que quebra uma regra abaixo. As páginas que você nomeia são conferidas seja qual for o estado, então dá para conferir uma antes de ela estar atual. |
| `python3 tools/translations.py stamp --lang de editor/brushes.md` | Define o `source` como a página em inglês de agora e não mexe em mais nada. |

**Traduzindo ou atualizando uma página.** Quatro passos:

1. Rode `prep` para a página. Para uma página desatualizada ele mostra também o diff do inglês desde o `source` antigo.
2. Escreva a página em `docs/translations/<code>/`, com o front matter que o `prep` imprimiu primeiro. Mantenha a
    estrutura do inglês linha por linha: mesmos títulos, listas, imagens, links, alertas e blocos `td-mcp`. Traduza o texto, o
    texto alternativo das imagens e o texto dos links, mais nada, no tom do idioma, veja abaixo.
3. Rode `check` para a página e corrija o que ele apontar até ficar limpo.
4. Rode `python3 tools/build_site.py`, `python3 tools/check_site.py` e `python3 tools/translations.py check`, depois
    faça commit das páginas de um idioma juntas.

Use `stamp` só quando a mudança no inglês não exige nada na tradução, como um erro de digitação, o alvo de um link que mudou ou uma
reescrita que diz a mesma coisa. Ele não olha a página, então leia antes o diff do inglês. Uma mudança no que a página
diz exige editar a tradução, e o novo `source` vai junto.

**Tom.** Todo idioma mantém o tom casual do [getting-started.md](getting-started.md), na forma que a documentação de software
usa ali:

| Idioma | Tratamento |
| --- | --- |
| de | du, minúsculo |
| fr | vous |
| es | tú, em espanhol neutro, ustedes no plural como diz o README do catálogo |
| pt | você, português do Brasil |
| ru | вы, minúsculo |
| zh-Hans | 你 |
| ja | forma です/ます, sem pronome |

**O que o `check` exige.** As regras valem para toda tradução atual, e o `check` roda antes do build no CI.

* Os ids dos títulos continuam sendo os do inglês. Todo título, o do topo também, leva o id do título em inglês,
    `## Größe ändern { #reshape }`, na mesma ordem e com os mesmos sufixos de repetição (`{ #inputs_1 }`). O Zensical
    descarta letras fora do ASCII ao fazer um id, e os links de F1 do editor, os links entre páginas, o glossário
    e o botão de idioma dependem dos ids em inglês.
* O código continua em inglês e byte por byte. Blocos de código são copiados com os comentários, e todo trecho de código
    em linha da página em inglês aparece na tradução. Classnames de entidades, chaves, Inputs e Outputs, nomes de ferramentas MCP, JSON, nomes
    de arquivo, nomes de teclas (`Ctrl`, `F1`) e nomes de produtos (TrenchDot, Godot, MCP, Live Link) também não são traduzidos.
* Links e imagens mantêm os alvos, só o texto do link e o texto alternativo mudam. As capturas de tela são as
    do inglês, uma página traduzida aponta para os mesmos arquivos, então o site publicado guarda uma cópia de cada.
* Os rótulos são os do editor. Um trecho em negrito, ou em itálico com ` > ` dentro, cujas partes são todas chaves do
    catálogo `crates/td_editor/locales/<code>.json`, precisa aparecer na página traduzida como o catálogo o escreve, as partes
    unidas com ` > `. Escreva o rótulo como o editor o mostra, sem flexionar, e monte a frase em volta dele. Um rótulo
    que não está no catálogo, como um botão do Godot ou do addon, continua em inglês.
* Alertas, blocos `td-mcp` e o título `## MCP` deles ficam onde estão. Escreva o marcador do alerta sozinho na linha,
    `> [!NOTE]`, o build coloca o título traduzido depois dele. Os blocos de GDScript mantêm a marca `gdscript`.
* Nada de travessão, meia-risca, hífen com espaços ou emoji, as regras das páginas em inglês valem em todo idioma.
* Em chinês e japonês cada parágrafo, item de lista e célula de tabela é uma linha de origem. Uma quebra de linha entre dois
    caracteres CJK aparece como um espaço no navegador, então o `check` a aponta. A pontuação de largura total é permitida, mas
    os caracteres de traço duplo U+2014 e U+2015 contam como travessões e não são permitidos.

**O glossário.** O `GLOSSARY.md` é traduzido como qualquer página, e cada termo mantém o seu id em inglês,
`## Aktivator { #activator }`. Um idioma que flexiona um termo lista as outras formas em `data-forms`,
`{ #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }`, para o cartão de hover achá-las também.
Um termo que é um nome de interface usa a palavra do catálogo, e o `check` as compara. Uma página traduzida usa um termo na
forma do próprio glossário onde ele aparece primeiro, é isso que a liga ao cartão. Quando você adiciona um termo ao glossário
em inglês, adicione-o aos traduzidos se puder, até lá o glossário de cada idioma fica em inglês.

## CI { #ci }

O CI pula os pushes na `main` que só mudam a documentação ou arquivos Markdown. Todo outro push verde na `main` substitui a
[beta contínua](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) por downloads novos do editor, do addon, da demo e do conteúdo.

O editor baixa o addon e o conteúdo só das releases do GitHub. O itch.io é uma vitrine que recebe os mesmos
arquivos nos seus próprios canais, `addon`, `nature-pack`, `demo-content` e assim por diante. A beta contínua é a única build,
então ela abastece esses canais simples.

| Zip na release | Instalado por |
| --- | --- |
| `func_godot-trenchdot-addon.zip` | *Godot > Instalar ou atualizar addon…* |
| `trenchdot-nature-pack.zip` | *Godot > Adicionar conteúdo ao projeto…* |
| `trenchdot-demo-content.zip` | *Godot > Adicionar conteúdo ao projeto…* |

As pastas que cada zip guarda estão listadas em
[content.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/content.rs) e
[addon_install.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/addon_install.rs), e testes
conferem os comandos de zip do `ci.yml` com elas.

O editor acha os zips pela API do GitHub. O CI o compila com `TRENCHDOT_CHANNEL=beta`, então ele procura primeiro na
beta contínua. Uma build local tenta primeiro uma release com o nome da própria versão, depois cai na beta. Para
experimentar um zip antes de ele ser publicado, aponte o editor para outro servidor com a variável de ambiente
`TRENCHDOT_RELEASES_API`.

O CI também carimba cada beta, em `TRENCHDOT_BUILD` para o editor e no `plugin.cfg` do addon zipado, como
`0.1.0-beta.202611041532.d194b57`: o número da release, a hora UTC do commit e o commit. É assim que o editor
distingue um addon de uma beta mais antiga e oferece a atualização. Uma build local mantém a versão simples.

<div class="td-mcp" markdown>

## MCP { #mcp }

Vai contribuir com um agente de IA? O [AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md) lista as regras
e as verificações que um pull request precisa passar.

Os testes e2e controlam o editor pelo [servidor MCP](mcp.md), e `example_mcp_scripts_replay` reproduz todo script em
`examples/mcp`.

Para controlar um editor seu pelo shell:

```sh
trenchdot --mcp-http=7850 --default-prefs &
TRENCHDOT_MCP_URL=http://127.0.0.1:7850/mcp python3 tools/mcp_client.py call get_state
```

Defina `TRENCHDOT_MCP_TOKEN` com o mesmo valor nos dois para experimentar com um [token](mcp.md#the-http-token).

### MCP na documentação { #mcp-in-the-docs }

O interruptor **MCP** no cabeçalho, desligado por padrão, esconde tudo sobre MCP para as páginas continuarem legíveis para quem
nunca mexe nele. Em uma página, ponha todo o texto de MCP em uma só seção `## MCP` no fim de tudo, dentro de um bloco `td-mcp`:

```html
<div class="td-mcp" markdown>

## MCP

What only MCP users need.

</div>
```

Mantenha as linhas em branco em volta do conteúdo, o GitHub só lê o Markdown lá dentro com elas. Páginas só sobre MCP ficam
na parte **MCP** do `SUMMARY.md`, que o interruptor também esconde.

</div>
