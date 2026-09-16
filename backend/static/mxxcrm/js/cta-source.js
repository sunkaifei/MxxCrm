 sensors.quick('isReady',function(){
	 	
	 	var prolist=sensors.getPresetProperties();
			
			//推广渠道
			if(prolist.hasOwnProperty('$latest_utm_source')){
				
				var utm_source=prolist.$latest_utm_source;
				
				
		             
				
				
				
				
			}else{
				
				var utm_source=prolist.$latest_referrer_host;
				
				console.info(utm_source);
				//判断前向域名www.baidu.com
				if(utm_source!='' && utm_source!='undefined'){
					if(utm_source=="www.baidu.com"){
						utm_source="百度";
					}else if(utm_source=="www.so.com"){
						utm_source="360";
					}else if(utm_source=="www.sogo.com"){
						utm_source="搜狗";
					}else{
						utm_source="其他";
					}
				}else{
					utm_source='';
				}
			}
		
			//推广方式
			
		    if(prolist.hasOwnProperty('$latest_utm_medium')){
				
				var utm_medium=prolist.$latest_utm_medium;
				
				
				
				
				
			}else{
				
				//推广方式
				var utm_medium='SEO';
			
			
			}
			//推广计划
			
			if(prolist.hasOwnProperty('$latest_utm_campaign')){
				
				var utm_campaign=prolist.$latest_utm_campaign;
			}else{
				var utm_campaign='';
			}
			 
			 
			 
			//推广单元
						
			if(prolist.hasOwnProperty('$latest_utm_content')){
				
				var utm_content=prolist.$latest_utm_content;
			}else{
				var utm_content='';
			}
			//关键词
			
			if(prolist.hasOwnProperty('$latest_utm_term')){
				
				var utm_term=prolist.$latest_utm_term;
			}else{
				//关键词
			var utm_term=prolist.$latest_search_keyword;
			}
			
			
			//关键词
			
			if(prolist.hasOwnProperty('$latest_search_keyword')){
				
				var search_keyword=prolist.$latest_search_keyword;
			}else{
				//关键词
			var search_keyword="";
			}
			
			
			
			
			
			      
						//循环推广渠道
						/*if(utm_source!=''){
							for(var i=0;i<qudaodata.length;i++){
								
								if(qudaodata[utm_source]==utm_source){
									var new_utm_source=qudaodata[i].value;
									break
								}else{
									var new_utm_source="";
								}
							}
							
							var utm_source=new_utm_source;
						}*/
		
						//循环推广方式
						/*if(utm_medium!=''){
							for(var i=0;i<fangshidata.length;i++){
								if(fangshidata[i].label==utm_medium){
									var new_utm_medium=fangshidata[i].value;
									break
								}else{
									
									var new_utm_medium="";
								}
							}
							
							var utm_medium=new_utm_medium;
						}*/
						
						
						$.cookie("utm_source",utm_source);
						$.cookie("utm_medium",utm_medium);
						$.cookie("utm_campaign",utm_campaign);
						$.cookie("utm_content",utm_content);
						$.cookie("utm_term",utm_term);
						//搜索词
						$.cookie("search_keyword",search_keyword);
	
	
	 	
	 	
	 	
	 	
	 	
	 });
	