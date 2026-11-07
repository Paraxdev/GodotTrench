---
source: cb5ad8fa1eb282094d0cdec538d72382b939f070
---

# Pré-calculando a iluminação { #baking-lighting }

Pré-calcular resolve a luz do sol, do céu, das suas lâmpadas e dos materiais brilhantes uma só vez, com as reflexões incluídas, e a guarda no mapa como um lightmap. O Godot então desenha essa luz em vez de calculá-la a cada quadro. Você ganha luz de rebote macia, vazamento colorido das lâmpadas e cantos mais escuros quase sem custo na execução. As luzes que você quer ligar, desligar ou mover continuam em tempo real ao lado do pré-cálculo.

![Esquerda: uma sala na Prévia iluminada, cinza chapado com manchas de sol de borda dura sob as janelas. Direita: a mesma sala depois de pré-calcular, luz quente de lâmpada, cantos mais escuros e manchas de sol macias](../assets/light-baking/lit-preview-and-baked.png)

O pré-cálculo cobre a geometria estática: brushes do mundo, malhas, terrenos, displacements e entidades de brush que não se movem. Entidades de brush que se movem, como portas e plataformas, volumes de trigger, decals, texturas de ferramenta e camadas deixadas de fora da exportação não são iluminados pelo pré-cálculo nem projetam sombras pré-calculadas.

## Pré-calculando um mapa { #baking-a-map }

1. Abra a janela Pré-calcular iluminação

    Escolha *Godot > Pré-calcular iluminação…*.

    ![O menu Godot com Pré-calcular iluminação sob o mouse e sua dica aparecendo](../assets/light-baking/menu-bake-lighting.png)

2. Confira o que entra

    A linha de cima resume, por exemplo "Pré-calcula o sol, 1 luz, o céu e os materiais brilhantes em 102 superfícies", e diz quais luzes continuam em tempo real.

    ![A janela Pré-calcular iluminação com Qualidade em Médio, Tamanho do texel 16,0 u, Suavidade das sombras 0,30 e Executa em CPU](../assets/light-baking/bake-dialog.png)

3. Escolha uma qualidade e um tamanho de texel

    Veja [as opções](#dialog-options) abaixo. Os padrões servem para uma primeira olhada.

4. Clique em **Pré-calcular**

    Uma janela de progresso mostra a etapa, e você pode continuar editando enquanto isso. As mudanças feitas durante o pré-cálculo não entram nele. **Cancelar** o interrompe e o mapa mantém a iluminação que tinha.

5. Veja o resultado

    Quando termina, a vista 3D passa para o sombreamento **Iluminação pré-calculada**, e a barra de status diz quanto tempo levou e qual o tamanho do lightmap.

O pré-cálculo é salvo dentro do `.tdm`, e pré-calcular é um único passo de desfazer. **Remover pré-cálculo** na mesma janela o descarta, e o Godot volta a iluminar o mapa em tempo real.

## Opções da janela { #dialog-options }

| Opção | O que faz |
| --- | --- |
| Qualidade | Quantos raios e rebotes o rastreador gasta, veja a tabela abaixo |
| Tamanho do texel | Unidades de mapa que um texel do lightmap cobre, de 2 a 128, 16 (50 cm) por padrão. Menor é mais nítido e mais lento |
| Suavidade das sombras | De 0 a 1. Espalha as luzes que não têm um `light_size` próprio para as sombras terem bordas macias. 0 as mantém duras |
| Executa em | CPU, GPU ou CPU + GPU, veja abaixo |

| Qualidade | Raios por texel | Rebotes | Raios de sombra por luz | Use para |
| --- | --- | --- | --- | --- |
| Pré-visualização | 32 | 1 | 4 | Posicionar luzes, ruidoso mas rápido |
| Médio | 96 | 2 | 8 | O trabalho do dia a dia, o padrão |
| Alto | 192 | 3 | 12 | Um resultado limpo |
| Final | 384 | 4 | 16 | A versão que você publica, lento em mapas grandes |

O lightmap tem no máximo 4096 texels por lado. Quando um mapa não cabe no tamanho de texel que você pediu, o pré-cálculo usa um mais grosso e avisa na barra de status. O mapa lembra as opções com que foi pré-calculado pela última vez.

### CPU, GPU ou os dois { #cpu-gpu-or-both }

| Executa em | Escolha quando |
| --- | --- |
| CPU | Sempre funciona, em todos os núcleos. O padrão seguro |
| GPU | A máquina tem uma placa de vídeo dedicada, que costuma ser bem mais rápida |
| CPU + GPU | As duas são fortes. A placa e todos os núcleos pegam trabalho de uma fila só, cada um o quanto aguenta |

Quando a GPU não pode ser usada, por exemplo porque não existe, é um renderizador por software ou o mapa é grande demais para os buffers dela, a CPU pré-calcula o mapa e a barra de status diz o motivo. Uma GPU que falha no meio passa o resto para a CPU.

## Olhando o pré-cálculo { #looking-at-the-bake }

*Exibir > Sombreamento > Iluminação pré-calculada* mostra o pré-cálculo, e quando o mapa tem um, F4 o inclui ao percorrer os sombreamentos. *Exibir > Sombreamento > Vista pré-calculada* escolhe o que ele mostra.

![Exibir > Sombreamento > Vista pré-calculada aberto, com Texturas com luz pré-calculada marcado e Só lightmap sob o mouse](../assets/light-baking/menu-baked-view.png)

| Vista pré-calculada | Mostra |
| --- | --- |
| Texturas com luz pré-calculada | As texturas iluminadas pelo lightmap, perto do que o Godot mostra |
| Só lightmap | Só a luz, sem texturas, para julgar o rebote e o ruído |
| Máscara de sombra do sol | Quanto do sol chega a cada ponto, branco sob sol pleno |
| Oclusão de ambiente | Escurecimento nos cantos e embaixo dos objetos, só para conferir, o Godot não usa |

![A mesma sala nas quatro vistas pré-calculadas: texturas com luz pré-calculada em cima à esquerda, só o lightmap em cima à direita, a máscara de sombra do sol embaixo à esquerda e a oclusão de ambiente embaixo à direita](../assets/light-baking/baked-views.png)

## Chaves de luz { #light-keys }

Toda `light` e `light_spot` tem uma chave `bake_mode`, e o worldspawn tem `sun_bake_mode` para o sol:

| Valor | No pré-cálculo | No Godot |
| --- | --- | --- |
| `auto` | Como `baked`, mas uma luz com `targetname` ou `start_on` 0 continua em tempo real, já que a E/S pode ligá-la e desligá-la. O sol é sempre pré-calculado | Estática ou desativada |
| `baked` | Luz direta e de rebote e as sombras dela | Estática, o lightmap a carrega |
| `bounce` | Só a luz de rebote | Dinâmica, o Godot desenha a luz direta e as sombras dela em tempo real |
| `realtime` | Deixada de fora | Desativada, totalmente em tempo real |

Use `bounce` para uma lâmpada cujas sombras precisam seguir objetos em movimento enquanto o rebote dela ainda enche a sala. Os nomes do próprio Godot `static`, `dynamic` e `disabled` também funcionam. Uma luz em tempo real também ilumina objetos em movimento pelas [sondas de luz](../godot/baked-lighting.md#light-probes).

`light_size` é o raio da lâmpada em metros. Uma lâmpada maior projeta sombras mais macias, e 0 usa a suavidade das sombras da janela. `light_indirect_energy` escala a luz de rebote, e `omni_attenuation`, `spot_attenuation` e `spot_angle_attenuation` moldam a atenuação. A luz do Godot recebe todas elas como propriedades de mesmo nome, para a luz em tempo real combinar com o pré-cálculo.

## Quando um pré-cálculo fica desatualizado { #when-a-bake-goes-out-of-date }

O pré-cálculo lembra a geometria e as luzes de que foi feito. Mover, remodelar, adicionar ou apagar geometria estática, ou mudar uma luz, uma sonda de luz, as chaves de sol e céu do worldspawn, as texturas em uso ou as configurações delas, o deixa desatualizado. A janela Pré-calcular iluminação então avisa que o mapa mudou desde o último pré-cálculo.

![A janela Pré-calcular iluminação com a linha amarela "O mapa mudou desde o último pré-cálculo. Faça de novo para atualizar a iluminação." e os botões Pré-calcular e Remover pré-cálculo](../assets/light-baking/bake-dialog-out-of-date.png)

Até você pré-calcular de novo, as faces que mudaram mostram a Prévia iluminada no editor e a luz média do mapa no Godot, enquanto o resto mantém a luz pré-calculada. Aqui o pilar foi movido depois do pré-cálculo: ele mostra a Prévia iluminada, e a sombra antiga dele continua no chão. Pré-calcule de novo antes de publicar.

![A sala pré-calculada depois de mover o pilar, o pilar está iluminado por igual e a sombra pré-calculada antiga dele fica onde ele estava](../assets/light-baking/bake-out-of-date.png)

## Configurações de textura { #texture-settings }

Selecione uma face e abra **Config. de textura** no Inspetor para definir como toda face com aquela textura é tratada, no mapa inteiro. As configurações são salvas com o mapa.

![O Inspetor com uma face selecionada e Config. de textura aberto: Pré-calcular luz nela, Projeta sombras pré-calculadas, Detalhe do lightmap, Repetir a cada e Projeção](../assets/light-baking/texture-settings.png)

| Configuração | O que faz | Use para |
| --- | --- | --- |
| Pré-calcular luz nela | Desligado deixa as faces fora do lightmap, o Godot dá a elas a luz média do mapa | Céu, água, faces que ninguém vê de perto |
| Projeta sombras pré-calculadas | Desligado deixa a luz atravessar as faces no pré-cálculo | Vidro, grades, decals finos |
| Detalhe do lightmap | Multiplica o tamanho do texel nessas faces, abaixo de 1 é mais nítido, acima de 1 economiza espaço | Sombras nítidas num piso, borradas num teto distante |
| Repetir a cada | Unidades de mapa que uma repetição da textura cobre, em vez do tamanho em pixels dela | Uma escala só para uma textura em todo lugar |
| Projeção | Mundo ignora o deslocamento, a escala e a rotação de cada face, então a textura segue sem emendas | Papel de parede, azulejos, concreto |

> [!TIP]
> Para um mapa no estilo backrooms, defina o papel de parede com a projeção Mundo e um tamanho de repetição uma única vez, e toda parede que você desenhar com ele se alinha sem tocar na ferramenta Textura.

Uma face com as duas opções de pré-cálculo desligadas fica totalmente fora do pré-cálculo. O tamanho de repetição e a projeção também valem no Godot.

<div class="td-mcp" markdown>

## MCP { #mcp }

A ferramenta `bake_lighting` pré-calcula com as mesmas opções da janela, `quality`, `texel_size`, `softness` e `backend` (`cpu`, `gpu` ou `hybrid`), e `view` troca a Vista pré-calculada. Ela espera o pré-cálculo terminar e devolve o tamanho do lightmap, as contagens, a GPU usada ou o motivo de não ter sido, e `out_of_date`. A op `settings` da ferramenta `texture` lê e define as configurações de textura, com `material`, `bake`, `casts`, `texel_scale`, `size` (`null` a limpa) e `projection` (`face` ou `world`). Veja [Servidor MCP](../mcp.md).

</div>
