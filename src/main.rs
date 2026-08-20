

const G_RATIO: f32 = 1.618;
const FIB_HASH_MULT:u32 = 2654435761;
const MAX_OFFSET:usize = 65535;
const MINMATCH:usize = 4;
const HEADER_SIZE: usize = 4;

//_______________________________________descompactacao/decompress_____________________________
// codigo

fn decompressor(bytes: &Vec<u8>) -> Vec<u8>{
    let mut decompressed: Vec<u8> = Vec::new();

    let mut idx: usize = HEADER_SIZE;
    
    while true{
        if idx>=bytes.len(){
            //flush, etc
            break
        }

        let token_position = idx;
        let token = bytes[token_position];
        
        let mut literal_count: u128 =  ((token >> 4) &0x0F) as u128;
        let mut match_len: usize = (token &0x0F) as usize;
        
        let mut offset:usize;
        
        let mut extra_bytes: usize = 0;
        
        if literal_count >= 15{
            extra_bytes += 1;

            while ((bytes[idx+extra_bytes]) as u16) >= 255{
                literal_count += bytes[idx+extra_bytes] as u128;
                extra_bytes += 1;
            }
            
            literal_count += bytes[idx+extra_bytes] as u128;
        }
        
        let mut m_extra_bytes: usize = 0;
        
        if bytes.len() > idx + extra_bytes + 2 +literal_count as usize{
            let l: u16 = bytes[idx + extra_bytes + 1 + literal_count as usize] as u16;
            let h: u16 = bytes[idx + extra_bytes + 2 + literal_count as usize] as u16;
            offset = (l | (h << 8)) as usize;
        }else{
            offset = 0;
        }
        
        
        //extensao de matching
        if match_len >= 15{
            m_extra_bytes +=1; 
        }

        while (bytes[idx + extra_bytes + (literal_count as usize) + m_extra_bytes + 2]) as usize >=255{
            match_len+=255;
            m_extra_bytes += 1;
        }
        
        for i in (idx+extra_bytes+1..idx+extra_bytes+(literal_count as usize) + 1){
            decompressed.push(bytes[i]);
        }
        
        if offset > 0{
            let start = decompressed.len() - offset;
            for i in 0 .. (match_len + MINMATCH){
                let byte = decompressed[start+i];
                decompressed.push(byte);

            }
        }
        idx = idx + literal_count as usize + 3 + extra_bytes + m_extra_bytes;
    }

    decompressed
}





//hashing
fn fib_hashing(bytes: &[u8], p:usize, hash_bits: u32) -> usize { 
    let seq: u32 = u32::from_le_bytes([bytes[p], bytes[p+1], bytes[p+2], bytes[p+3]]);
    
    let h: u32 = seq.wrapping_mul(FIB_HASH_MULT);
    (h >> (32 - hash_bits)) as usize
}



//_______________________________________compressao/compactacao________________________________
fn compress(bytes: &[u8]) -> Vec<u8>{

    let mut compressed: Vec<u8> = Vec::new();
    
    let hash_bits:u32 = 16;
    let table_size: usize = 1 << hash_bits;
    
    let mut table: Vec<Option<usize>> = vec![None;table_size];
    
    let mut idx:usize = 0;
    let mut idx_end:usize = idx + MINMATCH;

    let mut literal_count:u128 = 0;

    let mut token_position: usize = 0;
    
    println!("Iniciando compressão");
    
    //loop de compressao
    while true{

        if idx_end >= bytes.len(){
            
            //flush/verificacao final
            if token_position < bytes.len(){
                println!("Flushing bytes finais");

                let mut token: u8;

                literal_count = (bytes[token_position ..].len()) as u128;
                
                if literal_count >= 15 {
                    
                    token = ((15& 0x0F) << 4) | (0 & 0x0F);
                    
                    compressed.push(token);
                    
                    literal_count -= 15;
                }
                while literal_count >= 255 {
                    compressed.push(255);
                    literal_count -= 255;
                }
                
                compressed.push(literal_count as u8);
                
                for i in token_position .. bytes.len(){
                    compressed.push(bytes[i]);
                }
                    
            }

            break;
        }


        //hashing direto no loop para evitar problemas de borrow/copia com os bytes
        let seq: u32 = u32::from_le_bytes([bytes[idx], bytes[idx+1], bytes[idx+2], bytes[idx+3]]);
        let h: u32 = seq.wrapping_mul(FIB_HASH_MULT);

        let hash_idx: usize = (h >> (32 - hash_bits)) as usize;
        
        let mut match_idx: usize;
        let mut match_idx_end: usize;

        let mut b_match: bool = false;

        match table[hash_idx]{
            Some(usize_t) => {
                
                match_idx = usize_t;
                match_idx_end = match_idx + MINMATCH;
                
                
                if idx-match_idx <= MAX_OFFSET{
                
                    while match_idx_end < idx && bytes[match_idx .. match_idx_end] == bytes[idx .. idx_end] && match_idx_end < bytes.len() && idx_end < bytes.len(){
                        b_match = true;
                        match_idx_end += 1;
                        idx_end +=1;
                    }
                }

            },
            None => {
                match_idx = 0;
                match_idx_end = 0;
                ()
            }
            
        }
        //match end
        table[hash_idx] = Some(idx);

        //bateu, montar token e checar possíveis expansões
        if b_match==true{
            
            let offset: u16 = (idx - match_idx) as u16;

            let mut token: u8;
            
            let mut match_len: usize = match_idx_end - match_idx - MINMATCH;
            token_position = match_len + MINMATCH;
            idx += match_len + MINMATCH;
            idx_end = idx + MINMATCH;

            //checar para expansoes de literais
            if literal_count >= 15 {

                token = ((15& 0x0F) << 4);
                literal_count-=15;

            }else{

                token = ((literal_count as u8 & 0x0F) << 4);

            }


            if match_len>=15{

                token = (token& 0xF0) | 15;

            }else{

                token = (token& 0xF0) | match_len as u8;

            }
            
            compressed.push(token);

            
            //expansao de literais
            while literal_count >= 255{
                compressed.push(255);
                literal_count -= 255;
            }
            compressed.push(literal_count as u8);

            //for loop para evitar problemas de borrow
            for i in token_position .. idx{
                compressed.push(bytes[i]);
            }
            
            
            compressed.extend_from_slice(&offset.to_le_bytes());


            //expansao de match
            while match_len>=255{
                compressed.push(255);
                match_len -= 255;
            }
            compressed.push(match_len as u8);

            
        }else{

            literal_count += 1;
            idx += 1;
            idx_end = idx + MINMATCH;

        }

    }
    
    return compressed; 

}




fn main(){
    
    let args: Vec<String> = std::env::args().collect();

    //if args[1] == "decompress"{
    //    decompress_file(&args[2]);
    //    std::process::exit(0);
    //}

    let file_name : &String = &args[1];
    

    //let header = read_file_extension(file_name);    

    let arquivo =std::fs::read(&args[1]);
    let dados = match arquivo{
        Ok(T) => T,
        Err(err) =>{
            println!("Erro ao ler bytes do arquivo");
            return ();
        }
    };

    let mut saida:Vec<u8> = compress(&dados);
    

    //salvando arquivo
    let path = std::path::Path::new(file_name);
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let nome_saida = parent.join(format!("{}.tzp", stem));

    match std::fs::write(&nome_saida, &saida){
        Ok(_) => println!("Arquivo salvo em {}", nome_saida.display()),
        Err(err) => println!("Erro ao salvar arquivo: {}", err),
    }
    println!("Compressão finalizada | Hora: {:?} ", std::time::SystemTime::now());

}