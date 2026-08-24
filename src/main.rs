use indicatif::{ProgressBar,ProgressStyle};

const FIB_HASH_MULT:u32 = 2654435761;
const MAX_OFFSET:usize = 65535;
const MINMATCH:usize = 4;
const HEADER_SIZE: usize = 4;

//_______________________________________descompactacao/decompress_____________________________
fn decompress_file(file_name: &String){
    println!("Iniciando descompressão...");
        
    let file = std::fs::read(file_name);
    
    let bytes = match file{
        Ok(bytes_compressed) => bytes_compressed,
        Err(err) =>{
            println!("Erro durante a leitura do arquivo");
            return;
        }
    };

    let extension = String::from_utf8_lossy(
        &bytes[0..HEADER_SIZE].iter().cloned().take_while(|&b| b != 0).collect::<Vec<u8>>()
    ).into_owned();
    
    let decompressed = decompressor(&bytes);
    

    let path = std::path::Path::new(file_name);
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let exit_name = parent.join(format!("{}_.{}", stem, extension));

    match std::fs::write(&exit_name, decompressed){
        Ok(_) => println!("Arquivo salvo em {}", exit_name.display()),
        Err(err) => println!("Erro ao salvar arquivo: {}", err),
    }
    
}


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

            while (bytes[idx + extra_bytes + (literal_count as usize) + m_extra_bytes + 2]) as usize >=255{
                match_len+=255;
                m_extra_bytes += 1;
            }

            match_len += bytes[idx + extra_bytes + (literal_count as usize) + m_extra_bytes + 2] as usize;
        
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





//hashing (old)
//fn fib_hashing(bytes: &[u8], p:usize, hash_bits: u32) -> usize { 
//    let seq: u32 = u32::from_le_bytes([bytes[p], bytes[p+1], bytes[p+2], bytes[p+3]]);
//    
//    let h: u32 = seq.wrapping_mul(FIB_HASH_MULT);
//    (h >> (32 - hash_bits)) as usize
//}



//_______________________________________compressao/compactacao________________________________
fn compress(bytes: &[u8], header:&[u8], pb: &ProgressBar) -> Vec<u8>{

    let mut compressed: Vec<u8> = Vec::new();
    
    for i in 0 .. HEADER_SIZE{
        compressed.push(header[i]);
    }
    
    let hash_bits:u32 = 16;
    let table_size: usize = 1 << hash_bits;
    
    let mut table: Vec<usize> = vec![usize::MAX;table_size];
    
    let mut idx:usize = 0;
    let mut idx_end:usize = idx + MINMATCH;

    let mut literal_count:u128 = 0;

    let mut token_position: usize = idx;
    
    println!("Iniciando compressão");
    
    //loop de compressao
    while true{
        
        if idx % 16384 == 0{
            pb.set_position(idx as u64);
        }

        if idx_end >= bytes.len(){
            
            //flush/verificacao final
            if token_position < bytes.len(){
                println!("Flushing bytes finais");

                let mut token: u8;

                literal_count = (bytes[token_position ..].len()) as u128;
                
                if literal_count >= 15 {
                    
                    token = ((15& 0x0F) << 4) | (0 & 0x0F);
                    
                    literal_count -= 15;

                    compressed.push(token);

                    while literal_count >= 255 {
                        compressed.push(255);
                        literal_count -= 255;
                    }

                    if literal_count > 0 {

                        compressed.push(literal_count as u8);

                    }

                }else{

                    token = ((literal_count as u8 & 0x0F) << 4) | (0 & 0x0F);
                    compressed.push(token);

                }

                
                for i in token_position .. bytes.len(){
                    compressed.push(bytes[i]);
                }
                    
            }

            break;
        }


        //hashing direto no loop para evitar problemas de borrow/copia com os bytes
        let seq: u32 = u32::from_le_bytes(bytes[idx..idx+4].try_into().unwrap());
        let h: u32 = seq.wrapping_mul(FIB_HASH_MULT);

        let hash_idx: usize = (h >> (32 - hash_bits)) as usize;
        
        let mut match_idx: usize;
        let mut match_idx_end: usize;

        let mut b_match: bool = false;

        if table[hash_idx] != usize::MAX{

            match_idx = table[hash_idx];
            match_idx_end = match_idx + MINMATCH;
            
            
            if idx-match_idx <= MAX_OFFSET && bytes[match_idx .. match_idx_end] == bytes[idx .. idx_end]{
                
                // Mudando o b_match para ca (antes estava algumas linhas abaixo)
                // o comportamento (compressao e velocidade) muda.
                // Com o match abaixo (apos o laço WHILE) o codigo acaba
                // por buscar match de tamanho MINMATCH + 1, sendo melhor para
                // arquivos REPETITIVOS, pior para alta entropia.
                b_match = true;

                while match_idx_end < idx && match_idx_end < bytes.len() && idx_end < bytes.len() && bytes[match_idx_end] == bytes[idx_end]{

                    match_idx_end += 1;
                    idx_end += 1;

                }

                if b_match {
                    //match_idx_end -= 1;
                    //idx_end -= 1;
                    //b_match = true;
                }
            }

        } else{
            match_idx = 0;
            match_idx_end = 0;
        }
            
        //match end
        table[hash_idx] = idx;

        //bateu, montar token e checar possíveis expansões
        if b_match==true{
            
            let offset: u16 = (idx - match_idx) as u16;

            let mut token: u8;
            
            let mut match_len: usize = match_idx_end - match_idx - MINMATCH;
            let match_size = match_len + MINMATCH;

            let mut literal_saturated: bool = false;
            let mut match_saturated: bool = false;

            //checar para expansoes de literais
            if literal_count >= 15 {

                token = ((15& 0x0F) << 4);
                literal_count-=15;
                literal_saturated = true;

            }else{

                token = ((literal_count as u8 & 0x0F) << 4);
                literal_count = 0;

            }


            if match_len>=15{

                token = (token& 0xF0) | 15;
                match_len -= 15;
                match_saturated = true;

            }else{

                token = (token& 0xF0) | match_len as u8;
                match_len = 0;
            }
            
            compressed.push(token);

            
            //expansao de literais
            while literal_count >= 255{
                compressed.push(255);
                literal_count -= 255;

            }
            if literal_count>0 || literal_saturated == true{
                compressed.push(literal_count as u8);
            }

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
            if match_len > 0 || match_saturated == true{

                compressed.push(match_len as u8);

            }

            idx += match_size ;
            token_position = idx;
            idx_end = idx + MINMATCH;
            literal_count = 0;
            
        }else{

            literal_count += 1;
            idx += 1;
            idx_end = idx + MINMATCH;

        }

    }
    
    pb.finish_with_message("compressão concluída");
    println!("Tamanho final da saída {} | entrada {}  | Taxa de compressao: {} ", compressed.len(), bytes.len(), (1 as f32 - (compressed.len()) as f32/(bytes.len()) as f32) );

    return compressed; 
}


fn read_file_extension(file_name: &String) -> [u8;HEADER_SIZE]{
    let extensao = std::path::Path::new(file_name)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let mut header: [u8; 4] = [0; 4];
    for (i, b) in extensao.bytes().take(HEADER_SIZE).enumerate() {
        header[i] = b;
    }
    header
}


fn main(){
    
    let args: Vec<String> = std::env::args().collect();

    if args[1] == "decompress"{
        decompress_file(&args[2]);
        std::process::exit(0);
    }

    let file_name : &String = &args[1];

    let header: [u8;4]= read_file_extension(file_name);    

    let file :Result<Vec<u8>, std::io::Error>=std::fs::read(&args[1]);
    let bytes: Vec<u8> = match file{
        Ok(file_bytes) => file_bytes,
        Err(err) =>{
            println!("Erro ao ler bytes do arquivo");
            return ();
        }
    };

    let pb: ProgressBar = ProgressBar::new(bytes.len() as u64);
    pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
    ).unwrap());
    let compressed:Vec<u8> = compress(&bytes, &header, &pb);

    //salvando arquivo
    let path = std::path::Path::new(file_name);
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let exit_name= parent.join(format!("{}.tzp", stem));

    match std::fs::write(&exit_name, &compressed){
        Ok(_) => println!("Arquivo salvo em {}", exit_name.display()),
        Err(err) => println!("Erro ao salvar arquivo: {}", err),
    }
    println!("Compressão finalizada ");
    // | Hora: {:?} ", std::time::SystemTime::now());

}