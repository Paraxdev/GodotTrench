---
source: 320dd2516530d568c7e2acda31113f93ead3f05b
---

# Salvar e recuperar { #saving-and-recovery }

Aperte Ctrl+S para salvar, ou Ctrl+Shift+S para *Salvar como…*. As caixas de diálogo de arquivo começam na pasta do mapa atual, ou no projeto Godot quando o mapa nunca foi salvo, já que o addon carrega os mapas pelo caminho `res://`. *Arquivo > Mapas no projeto* lista todos os mapas do projeto, caso você só queira abrir um rápido.

## Backups e salvamento automático { #backups-and-autosave }

O editor guarda dois tipos de cópias reserva ao lado do seu mapa:

| Arquivo | Quando é escrito |
| --- | --- |
| `<map>.tdm.bak` | A cada salvamento, uma cópia da versão anterior. Renomeie para `.tdm` para voltar um salvamento |
| `<map>.tdm.autosave` | A cada três minutos, para toda aba com alterações não salvas |

Para mudar a frequência do salvamento automático, vá em *Arquivo > Preferências* e ajuste *Salvamento automático (minutos, 0 = desligado)*.

![O campo Salvamento automático (minutos, 0 = desligado) nas Preferências, com valor 3](../assets/saving/prefs-autosave.png)

Os salvamentos automáticos usam uma extensão que o Godot não importa, então um mapa pela metade nunca aparece no seu jogo. Salvar o mapa apaga o salvamento automático dele.

## Recuperar um mapa { #recovering-a-map }

Se o editor fechou antes de você salvar, abra o salvamento automático:

1. Escolha *Arquivo > Abrir mapa…*.
2. Escolha o tipo de arquivo **Salvamento automático (recuperar um mapa)** e abra o arquivo `.tdm.autosave`. A barra de status deve dizer que recuperou o mapa a partir do salvamento automático.
3. Salve. Isso escreve o `.tdm` de verdade de novo.

Mapas que você nunca salvou fazem o salvamento automático em um arquivo temporário. Você os encontra em *Arquivo > Abrir recente*, abaixo de *Mapas sem título salvos automaticamente*.

![Arquivo > Abrir recente com Nenhum mapa recente no topo e uma lista de mapas sem título salvos automaticamente abaixo](../assets/saving/open-recent-autosaves.png)

## Arquivos danificados { #damaged-files }

Um arquivo de mapa danificado no disco ainda abre. O editor lê todas as partes intactas, coloca os objetos cuja camada, grupo ou entidade se perdeu em uma camada chamada *Recovered*, e diz na barra de status o que não pôde ser lido. Ao salvar, o original danificado fica como `<map>.tdm.bak`, então você ainda pode restaurá-lo pelo controle de versão.
