

// Estrutura definida para o frame: [header][dados]
// header:  [TAMANHO DO ARQUIVO, EXTENSÃO ORIGINAL]
// corpo : [dados]

//dados -> logica do LZ4: sequencias de [sequence]
// cada sequence: [token][literals][offset] ou [token][literals]
// cada token tem 1 byte [nibble alto, nibble baixo]
// nibble alto: quantidade de literals 
// nibble baixo: tamanho do match - 4

// minmatch de 4 bytes/caracteres

const MINMATCH: usize = 4;

fn main() {
    
    let texto:String = String::from("Ola, computador, computador. Este e um texto de teste para testar o compressor manual do Clubix!. Qualquer semelhanca com outro compressor e mera coincidencia. computadores são legais!");
    let dados: &[u8] = texto.as_bytes();

    let mut saida:Vec<u8> = Vec::new();

    let mut p:usize = 0;
    let mut p_end:usize = MINMATCH;

    let mut token_position = 0;
    
    let mut token:u8 = 0;
    
    while p_end<dados.len(){
        
        println!(" Byte atual[{}..{}] -> {} ", p, p_end, std::str::from_utf8(&dados[p..p_end]).unwrap());

        let mut aux:usize = 0;
        let mut count:usize = 0;
        let mut count_end:usize = MINMATCH;

        while count_end<p{
            while true {
                if dados[count..count_end] == dados[p..p_end] && count_end<=p{
                    println!("Match encontrado -> [{}..{}] == [{}..{}] | {}", count, count_end, p, p_end, std::str::from_utf8(&dados[count..count_end]).unwrap());
                    count_end += 1;
                    p_end += 1;
    
                }else{
                    count_end-=1;
                    p_end=p+MINMATCH;
                    break;
                }
                
            }

            count = count_end ;
            count_end = count + MINMATCH;

        }
        

        p = p + 1;
        p_end = p + MINMATCH;
        
    }
    
}
