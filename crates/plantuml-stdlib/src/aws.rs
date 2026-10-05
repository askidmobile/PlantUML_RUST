//! AWS иконки (спрайты)
//!
//! Источник: репозиторий `awslabs/aws-icons-for-plantuml`, каталоги
//! `Compute`, `Storage`, `Database` и `NetworkingContentDelivery`.
//!
//! Спрайты хранятся в формате `[64x64/16z]` — том же сжатом формате,
//! что у kubernetes и Azure.
//!
//! ВНИМАНИЕ: в исходном репозитории часть файлов (30 из 248) содержит
//! ОБРЕЗАННЫЙ спрайт-заглушку: поток deflate в них неполный, а реальная
//! иконка лежит в PNG внутри `!function $ИМЯIMG`. Такие спрайты в модуль
//! не включены — иначе они молча отбрасывались бы при разборе.

use std::collections::HashMap;

/// Регистрирует включения AWS
pub fn register(registry: &mut HashMap<&'static str, &'static str>) {
    registry.insert("aws/Compute/AppRunner", AWS_AWS_COMPUTE_APPRUNNER);
    registry.insert("aws/Compute/AppRunner.puml", AWS_AWS_COMPUTE_APPRUNNER);
    registry.insert("aws/Compute/Batch", AWS_AWS_COMPUTE_BATCH);
    registry.insert("aws/Compute/Batch.puml", AWS_AWS_COMPUTE_BATCH);
    registry.insert("aws/Compute/Bottlerocket", AWS_AWS_COMPUTE_BOTTLEROCKET);
    registry.insert(
        "aws/Compute/Bottlerocket.puml",
        AWS_AWS_COMPUTE_BOTTLEROCKET,
    );
    registry.insert(
        "aws/Compute/ComputeOptimizer2",
        AWS_AWS_COMPUTE_COMPUTEOPTIMIZER2,
    );
    registry.insert(
        "aws/Compute/ComputeOptimizer2.puml",
        AWS_AWS_COMPUTE_COMPUTEOPTIMIZER2,
    );
    registry.insert("aws/Compute/DCV", AWS_AWS_COMPUTE_DCV);
    registry.insert("aws/Compute/DCV.puml", AWS_AWS_COMPUTE_DCV);
    registry.insert("aws/Compute/EC2AMI", AWS_AWS_COMPUTE_EC2AMI);
    registry.insert("aws/Compute/EC2AMI.puml", AWS_AWS_COMPUTE_EC2AMI);
    registry.insert(
        "aws/Compute/EC2AWSMicroserviceExtractorforNET",
        AWS_AWS_COMPUTE_EC2AWSMICROSERVICEEXTRACTORFORNET,
    );
    registry.insert(
        "aws/Compute/EC2AWSMicroserviceExtractorforNET.puml",
        AWS_AWS_COMPUTE_EC2AWSMICROSERVICEEXTRACTORFORNET,
    );
    registry.insert("aws/Compute/EC2AutoScaling", AWS_AWS_COMPUTE_EC2AUTOSCALING);
    registry.insert(
        "aws/Compute/EC2AutoScaling.puml",
        AWS_AWS_COMPUTE_EC2AUTOSCALING,
    );
    registry.insert("aws/Compute/EC2DBInstance", AWS_AWS_COMPUTE_EC2DBINSTANCE);
    registry.insert(
        "aws/Compute/EC2DBInstance.puml",
        AWS_AWS_COMPUTE_EC2DBINSTANCE,
    );
    registry.insert(
        "aws/Compute/EC2ElasticIPAddress",
        AWS_AWS_COMPUTE_EC2ELASTICIPADDRESS,
    );
    registry.insert(
        "aws/Compute/EC2ElasticIPAddress.puml",
        AWS_AWS_COMPUTE_EC2ELASTICIPADDRESS,
    );
    registry.insert(
        "aws/Compute/EC2ImageBuilder",
        AWS_AWS_COMPUTE_EC2IMAGEBUILDER,
    );
    registry.insert(
        "aws/Compute/EC2ImageBuilder.puml",
        AWS_AWS_COMPUTE_EC2IMAGEBUILDER,
    );
    registry.insert("aws/Compute/EC2Instance", AWS_AWS_COMPUTE_EC2INSTANCE);
    registry.insert("aws/Compute/EC2Instance.puml", AWS_AWS_COMPUTE_EC2INSTANCE);
    registry.insert("aws/Compute/EC2Instances", AWS_AWS_COMPUTE_EC2INSTANCES);
    registry.insert(
        "aws/Compute/EC2Instances.puml",
        AWS_AWS_COMPUTE_EC2INSTANCES,
    );
    registry.insert(
        "aws/Compute/EC2InstancewithCloudWatch",
        AWS_AWS_COMPUTE_EC2INSTANCEWITHCLOUDWATCH,
    );
    registry.insert(
        "aws/Compute/EC2InstancewithCloudWatch.puml",
        AWS_AWS_COMPUTE_EC2INSTANCEWITHCLOUDWATCH,
    );
    registry.insert("aws/Compute/EC2Rescue", AWS_AWS_COMPUTE_EC2RESCUE);
    registry.insert("aws/Compute/EC2Rescue.puml", AWS_AWS_COMPUTE_EC2RESCUE);
    registry.insert(
        "aws/Compute/EC2SpotInstance",
        AWS_AWS_COMPUTE_EC2SPOTINSTANCE,
    );
    registry.insert(
        "aws/Compute/EC2SpotInstance.puml",
        AWS_AWS_COMPUTE_EC2SPOTINSTANCE,
    );
    registry.insert(
        "aws/Compute/ElasticBeanstalk",
        AWS_AWS_COMPUTE_ELASTICBEANSTALK,
    );
    registry.insert(
        "aws/Compute/ElasticBeanstalk.puml",
        AWS_AWS_COMPUTE_ELASTICBEANSTALK,
    );
    registry.insert(
        "aws/Compute/ElasticFabricAdapter",
        AWS_AWS_COMPUTE_ELASTICFABRICADAPTER,
    );
    registry.insert(
        "aws/Compute/ElasticFabricAdapter.puml",
        AWS_AWS_COMPUTE_ELASTICFABRICADAPTER,
    );
    registry.insert(
        "aws/Compute/ElasticVMwareService",
        AWS_AWS_COMPUTE_ELASTICVMWARESERVICE,
    );
    registry.insert(
        "aws/Compute/ElasticVMwareService.puml",
        AWS_AWS_COMPUTE_ELASTICVMWARESERVICE,
    );
    registry.insert("aws/Compute/Lambda", AWS_AWS_COMPUTE_LAMBDA);
    registry.insert("aws/Compute/Lambda.puml", AWS_AWS_COMPUTE_LAMBDA);
    registry.insert(
        "aws/Compute/LambdaLambdaFunction",
        AWS_AWS_COMPUTE_LAMBDALAMBDAFUNCTION,
    );
    registry.insert(
        "aws/Compute/LambdaLambdaFunction.puml",
        AWS_AWS_COMPUTE_LAMBDALAMBDAFUNCTION,
    );
    registry.insert("aws/Compute/Lightsail", AWS_AWS_COMPUTE_LIGHTSAIL);
    registry.insert("aws/Compute/Lightsail.puml", AWS_AWS_COMPUTE_LIGHTSAIL);
    registry.insert(
        "aws/Compute/LightsailforResearch",
        AWS_AWS_COMPUTE_LIGHTSAILFORRESEARCH,
    );
    registry.insert(
        "aws/Compute/LightsailforResearch.puml",
        AWS_AWS_COMPUTE_LIGHTSAILFORRESEARCH,
    );
    registry.insert("aws/Compute/LocalZones", AWS_AWS_COMPUTE_LOCALZONES);
    registry.insert("aws/Compute/LocalZones.puml", AWS_AWS_COMPUTE_LOCALZONES);
    registry.insert("aws/Compute/NitroEnclaves", AWS_AWS_COMPUTE_NITROENCLAVES);
    registry.insert(
        "aws/Compute/NitroEnclaves.puml",
        AWS_AWS_COMPUTE_NITROENCLAVES,
    );
    registry.insert("aws/Compute/Outpostsfamily", AWS_AWS_COMPUTE_OUTPOSTSFAMILY);
    registry.insert(
        "aws/Compute/Outpostsfamily.puml",
        AWS_AWS_COMPUTE_OUTPOSTSFAMILY,
    );
    registry.insert("aws/Compute/Outpostsrack", AWS_AWS_COMPUTE_OUTPOSTSRACK);
    registry.insert(
        "aws/Compute/Outpostsrack.puml",
        AWS_AWS_COMPUTE_OUTPOSTSRACK,
    );
    registry.insert(
        "aws/Compute/Outpostsservers",
        AWS_AWS_COMPUTE_OUTPOSTSSERVERS,
    );
    registry.insert(
        "aws/Compute/Outpostsservers.puml",
        AWS_AWS_COMPUTE_OUTPOSTSSERVERS,
    );
    registry.insert(
        "aws/Compute/ParallelCluster",
        AWS_AWS_COMPUTE_PARALLELCLUSTER,
    );
    registry.insert(
        "aws/Compute/ParallelCluster.puml",
        AWS_AWS_COMPUTE_PARALLELCLUSTER,
    );
    registry.insert(
        "aws/Compute/ParallelComputingService",
        AWS_AWS_COMPUTE_PARALLELCOMPUTINGSERVICE,
    );
    registry.insert(
        "aws/Compute/ParallelComputingService.puml",
        AWS_AWS_COMPUTE_PARALLELCOMPUTINGSERVICE,
    );
    registry.insert(
        "aws/Compute/ServerlessApplicationRepository",
        AWS_AWS_COMPUTE_SERVERLESSAPPLICATIONREPOSITORY,
    );
    registry.insert(
        "aws/Compute/ServerlessApplicationRepository.puml",
        AWS_AWS_COMPUTE_SERVERLESSAPPLICATIONREPOSITORY,
    );
    registry.insert("aws/Compute/Wavelength", AWS_AWS_COMPUTE_WAVELENGTH);
    registry.insert("aws/Compute/Wavelength.puml", AWS_AWS_COMPUTE_WAVELENGTH);
    registry.insert("aws/Compute/all", AWS_AWS_COMPUTE_ALL);
    registry.insert("aws/Compute/all.puml", AWS_AWS_COMPUTE_ALL);
    registry.insert(
        "aws/Database/AuroraAmazonAuroraInstanceAlternate",
        AWS_AWS_DATABASE_AURORAAMAZONAURORAINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraAmazonAuroraInstanceAlternate.puml",
        AWS_AWS_DATABASE_AURORAAMAZONAURORAINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraAmazonRDSInstance",
        AWS_AWS_DATABASE_AURORAAMAZONRDSINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraAmazonRDSInstance.puml",
        AWS_AWS_DATABASE_AURORAAMAZONRDSINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraAmazonRDSInstanceAlternate",
        AWS_AWS_DATABASE_AURORAAMAZONRDSINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraAmazonRDSInstanceAlternate.puml",
        AWS_AWS_DATABASE_AURORAAMAZONRDSINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraMariaDBInstance",
        AWS_AWS_DATABASE_AURORAMARIADBINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraMariaDBInstance.puml",
        AWS_AWS_DATABASE_AURORAMARIADBINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraMariaDBInstanceAlternate",
        AWS_AWS_DATABASE_AURORAMARIADBINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraMariaDBInstanceAlternate.puml",
        AWS_AWS_DATABASE_AURORAMARIADBINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraMySQLInstance",
        AWS_AWS_DATABASE_AURORAMYSQLINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraMySQLInstance.puml",
        AWS_AWS_DATABASE_AURORAMYSQLINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraMySQLInstanceAlternate",
        AWS_AWS_DATABASE_AURORAMYSQLINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraMySQLInstanceAlternate.puml",
        AWS_AWS_DATABASE_AURORAMYSQLINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraOracleInstance",
        AWS_AWS_DATABASE_AURORAORACLEINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraOracleInstance.puml",
        AWS_AWS_DATABASE_AURORAORACLEINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraOracleInstanceAlternate",
        AWS_AWS_DATABASE_AURORAORACLEINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraOracleInstanceAlternate.puml",
        AWS_AWS_DATABASE_AURORAORACLEINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraPIOPSInstance",
        AWS_AWS_DATABASE_AURORAPIOPSINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraPIOPSInstance.puml",
        AWS_AWS_DATABASE_AURORAPIOPSINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraPostgreSQLInstance",
        AWS_AWS_DATABASE_AURORAPOSTGRESQLINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraPostgreSQLInstance.puml",
        AWS_AWS_DATABASE_AURORAPOSTGRESQLINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraPostgreSQLInstanceAlternate",
        AWS_AWS_DATABASE_AURORAPOSTGRESQLINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraPostgreSQLInstanceAlternate.puml",
        AWS_AWS_DATABASE_AURORAPOSTGRESQLINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraSQLServerInstance",
        AWS_AWS_DATABASE_AURORASQLSERVERINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraSQLServerInstance.puml",
        AWS_AWS_DATABASE_AURORASQLSERVERINSTANCE,
    );
    registry.insert(
        "aws/Database/AuroraSQLServerInstanceAlternate",
        AWS_AWS_DATABASE_AURORASQLSERVERINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraSQLServerInstanceAlternate.puml",
        AWS_AWS_DATABASE_AURORASQLSERVERINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/AuroraTrustedLanguageExtensionsforPostgreSQL",
        AWS_AWS_DATABASE_AURORATRUSTEDLANGUAGEEXTENSIONSFORPOSTGRESQL,
    );
    registry.insert(
        "aws/Database/AuroraTrustedLanguageExtensionsforPostgreSQL.puml",
        AWS_AWS_DATABASE_AURORATRUSTEDLANGUAGEEXTENSIONSFORPOSTGRESQL,
    );
    registry.insert("aws/Database/Database", AWS_AWS_DATABASE_DATABASE);
    registry.insert("aws/Database/Database.puml", AWS_AWS_DATABASE_DATABASE);
    registry.insert(
        "aws/Database/DatabaseMigrationService",
        AWS_AWS_DATABASE_DATABASEMIGRATIONSERVICE,
    );
    registry.insert(
        "aws/Database/DatabaseMigrationService.puml",
        AWS_AWS_DATABASE_DATABASEMIGRATIONSERVICE,
    );
    registry.insert(
        "aws/Database/DatabaseMigrationServiceDatabasemigrationworkflowjob",
        AWS_AWS_DATABASE_DATABASEMIGRATIONSERVICEDATABASEMIGRATIONWORKFLOWJOB,
    );
    registry.insert(
        "aws/Database/DatabaseMigrationServiceDatabasemigrationworkflowjob.puml",
        AWS_AWS_DATABASE_DATABASEMIGRATIONSERVICEDATABASEMIGRATIONWORKFLOWJOB,
    );
    registry.insert("aws/Database/DocumentDB", AWS_AWS_DATABASE_DOCUMENTDB);
    registry.insert("aws/Database/DocumentDB.puml", AWS_AWS_DATABASE_DOCUMENTDB);
    registry.insert(
        "aws/Database/DocumentDBElasticClusters",
        AWS_AWS_DATABASE_DOCUMENTDBELASTICCLUSTERS,
    );
    registry.insert(
        "aws/Database/DocumentDBElasticClusters.puml",
        AWS_AWS_DATABASE_DOCUMENTDBELASTICCLUSTERS,
    );
    registry.insert("aws/Database/DynamoDB", AWS_AWS_DATABASE_DYNAMODB);
    registry.insert("aws/Database/DynamoDB.puml", AWS_AWS_DATABASE_DYNAMODB);
    registry.insert(
        "aws/Database/DynamoDBAmazonDynamoDBAccelerator",
        AWS_AWS_DATABASE_DYNAMODBAMAZONDYNAMODBACCELERATOR,
    );
    registry.insert(
        "aws/Database/DynamoDBAmazonDynamoDBAccelerator.puml",
        AWS_AWS_DATABASE_DYNAMODBAMAZONDYNAMODBACCELERATOR,
    );
    registry.insert(
        "aws/Database/DynamoDBAttribute",
        AWS_AWS_DATABASE_DYNAMODBATTRIBUTE,
    );
    registry.insert(
        "aws/Database/DynamoDBAttribute.puml",
        AWS_AWS_DATABASE_DYNAMODBATTRIBUTE,
    );
    registry.insert(
        "aws/Database/DynamoDBAttributes",
        AWS_AWS_DATABASE_DYNAMODBATTRIBUTES,
    );
    registry.insert(
        "aws/Database/DynamoDBAttributes.puml",
        AWS_AWS_DATABASE_DYNAMODBATTRIBUTES,
    );
    registry.insert(
        "aws/Database/DynamoDBGlobalsecondaryindex",
        AWS_AWS_DATABASE_DYNAMODBGLOBALSECONDARYINDEX,
    );
    registry.insert(
        "aws/Database/DynamoDBGlobalsecondaryindex.puml",
        AWS_AWS_DATABASE_DYNAMODBGLOBALSECONDARYINDEX,
    );
    registry.insert("aws/Database/DynamoDBItem", AWS_AWS_DATABASE_DYNAMODBITEM);
    registry.insert(
        "aws/Database/DynamoDBItem.puml",
        AWS_AWS_DATABASE_DYNAMODBITEM,
    );
    registry.insert("aws/Database/DynamoDBItems", AWS_AWS_DATABASE_DYNAMODBITEMS);
    registry.insert(
        "aws/Database/DynamoDBItems.puml",
        AWS_AWS_DATABASE_DYNAMODBITEMS,
    );
    registry.insert(
        "aws/Database/DynamoDBStandardAccessTableClass",
        AWS_AWS_DATABASE_DYNAMODBSTANDARDACCESSTABLECLASS,
    );
    registry.insert(
        "aws/Database/DynamoDBStandardAccessTableClass.puml",
        AWS_AWS_DATABASE_DYNAMODBSTANDARDACCESSTABLECLASS,
    );
    registry.insert(
        "aws/Database/DynamoDBStream",
        AWS_AWS_DATABASE_DYNAMODBSTREAM,
    );
    registry.insert(
        "aws/Database/DynamoDBStream.puml",
        AWS_AWS_DATABASE_DYNAMODBSTREAM,
    );
    registry.insert("aws/Database/DynamoDBTable", AWS_AWS_DATABASE_DYNAMODBTABLE);
    registry.insert(
        "aws/Database/DynamoDBTable.puml",
        AWS_AWS_DATABASE_DYNAMODBTABLE,
    );
    registry.insert("aws/Database/ElastiCache", AWS_AWS_DATABASE_ELASTICACHE);
    registry.insert(
        "aws/Database/ElastiCache.puml",
        AWS_AWS_DATABASE_ELASTICACHE,
    );
    registry.insert(
        "aws/Database/ElastiCacheCacheNode",
        AWS_AWS_DATABASE_ELASTICACHECACHENODE,
    );
    registry.insert(
        "aws/Database/ElastiCacheCacheNode.puml",
        AWS_AWS_DATABASE_ELASTICACHECACHENODE,
    );
    registry.insert(
        "aws/Database/ElastiCacheElastiCacheforRedis",
        AWS_AWS_DATABASE_ELASTICACHEELASTICACHEFORREDIS,
    );
    registry.insert(
        "aws/Database/ElastiCacheElastiCacheforRedis.puml",
        AWS_AWS_DATABASE_ELASTICACHEELASTICACHEFORREDIS,
    );
    registry.insert(
        "aws/Database/ElastiCacheElastiCacheforValkey",
        AWS_AWS_DATABASE_ELASTICACHEELASTICACHEFORVALKEY,
    );
    registry.insert(
        "aws/Database/ElastiCacheElastiCacheforValkey.puml",
        AWS_AWS_DATABASE_ELASTICACHEELASTICACHEFORVALKEY,
    );
    registry.insert("aws/Database/Keyspaces", AWS_AWS_DATABASE_KEYSPACES);
    registry.insert("aws/Database/Keyspaces.puml", AWS_AWS_DATABASE_KEYSPACES);
    registry.insert("aws/Database/MemoryDB", AWS_AWS_DATABASE_MEMORYDB);
    registry.insert("aws/Database/MemoryDB.puml", AWS_AWS_DATABASE_MEMORYDB);
    registry.insert("aws/Database/Neptune", AWS_AWS_DATABASE_NEPTUNE);
    registry.insert("aws/Database/Neptune.puml", AWS_AWS_DATABASE_NEPTUNE);
    registry.insert(
        "aws/Database/OracleDatabaseatAWS",
        AWS_AWS_DATABASE_ORACLEDATABASEATAWS,
    );
    registry.insert(
        "aws/Database/OracleDatabaseatAWS.puml",
        AWS_AWS_DATABASE_ORACLEDATABASEATAWS,
    );
    registry.insert("aws/Database/RDS", AWS_AWS_DATABASE_RDS);
    registry.insert("aws/Database/RDS.puml", AWS_AWS_DATABASE_RDS);
    registry.insert(
        "aws/Database/RDSBlueGreenDeployments",
        AWS_AWS_DATABASE_RDSBLUEGREENDEPLOYMENTS,
    );
    registry.insert(
        "aws/Database/RDSBlueGreenDeployments.puml",
        AWS_AWS_DATABASE_RDSBLUEGREENDEPLOYMENTS,
    );
    registry.insert(
        "aws/Database/RDSOptimizedWrites",
        AWS_AWS_DATABASE_RDSOPTIMIZEDWRITES,
    );
    registry.insert(
        "aws/Database/RDSOptimizedWrites.puml",
        AWS_AWS_DATABASE_RDSOPTIMIZEDWRITES,
    );
    registry.insert(
        "aws/Database/RDSProxyInstance",
        AWS_AWS_DATABASE_RDSPROXYINSTANCE,
    );
    registry.insert(
        "aws/Database/RDSProxyInstance.puml",
        AWS_AWS_DATABASE_RDSPROXYINSTANCE,
    );
    registry.insert(
        "aws/Database/RDSProxyInstanceAlternate",
        AWS_AWS_DATABASE_RDSPROXYINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/RDSProxyInstanceAlternate.puml",
        AWS_AWS_DATABASE_RDSPROXYINSTANCEALTERNATE,
    );
    registry.insert(
        "aws/Database/RDSTrustedLanguageExtensionsforPostgreSQL",
        AWS_AWS_DATABASE_RDSTRUSTEDLANGUAGEEXTENSIONSFORPOSTGRESQL,
    );
    registry.insert(
        "aws/Database/RDSTrustedLanguageExtensionsforPostgreSQL.puml",
        AWS_AWS_DATABASE_RDSTRUSTEDLANGUAGEEXTENSIONSFORPOSTGRESQL,
    );
    registry.insert("aws/Database/Timestream", AWS_AWS_DATABASE_TIMESTREAM);
    registry.insert("aws/Database/Timestream.puml", AWS_AWS_DATABASE_TIMESTREAM);
    registry.insert(
        "aws/NetworkingContentDelivery/APIGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APIGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/APIGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APIGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/APIGatewayEndpoint",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APIGATEWAYENDPOINT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/APIGatewayEndpoint.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APIGATEWAYENDPOINT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMesh",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESH,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMesh.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESH,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshMesh",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHMESH,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshMesh.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHMESH,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualNode",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALNODE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualNode.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALNODE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualRouter",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALROUTER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualRouter.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALROUTER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualService",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALSERVICE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/AppMeshVirtualService.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALSERVICE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ApplicationRecoveryController",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPLICATIONRECOVERYCONTROLLER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ApplicationRecoveryController.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_APPLICATIONRECOVERYCONTROLLER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ClientVPN",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLIENTVPN,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ClientVPN.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLIENTVPN,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFront",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFront.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFrontDownloadDistribution",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTDOWNLOADDISTRIBUTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFrontDownloadDistribution.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTDOWNLOADDISTRIBUTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFrontEdgeLocation",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTEDGELOCATION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFrontEdgeLocation.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTEDGELOCATION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFrontStreamingDistribution",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTSTREAMINGDISTRIBUTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudFrontStreamingDistribution.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTSTREAMINGDISTRIBUTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMap",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAP,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMap.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAP,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMapNamespace",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPNAMESPACE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMapNamespace.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPNAMESPACE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMapResource",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPRESOURCE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMapResource.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPRESOURCE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMapService",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPSERVICE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudMapService.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPSERVICE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWAN",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWAN,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWAN.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWAN,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWANCoreNetworkEdge",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANCORENETWORKEDGE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWANCoreNetworkEdge.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANCORENETWORKEDGE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWANSegmentNetwork",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANSEGMENTNETWORK,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWANSegmentNetwork.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANSEGMENTNETWORK,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWANTransitGatewayRouteTableAttachment",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANTRANSITGATEWAYROUTETABLEATTACHMENT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/CloudWANTransitGatewayRouteTableAttachment.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANTRANSITGATEWAYROUTETABLEATTACHMENT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/DirectConnect",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_DIRECTCONNECT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/DirectConnect.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_DIRECTCONNECT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/DirectConnectGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_DIRECTCONNECTGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/DirectConnectGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_DIRECTCONNECTGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancing",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCING,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancing.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCING,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingApplicationLoadBalancer",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGAPPLICATIONLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingApplicationLoadBalancer.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGAPPLICATIONLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingClassicLoadBalancer",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGCLASSICLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingClassicLoadBalancer.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGCLASSICLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingGatewayLoadBalancer",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGGATEWAYLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingGatewayLoadBalancer.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGGATEWAYLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingNetworkLoadBalancer",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGNETWORKLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/ElasticLoadBalancingNetworkLoadBalancer.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGNETWORKLOADBALANCER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/GlobalAccelerator",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_GLOBALACCELERATOR,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/GlobalAccelerator.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_GLOBALACCELERATOR,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Interconnect",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_INTERCONNECT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Interconnect.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_INTERCONNECT,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/NetworkingContentDelivery",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_NETWORKINGCONTENTDELIVERY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/NetworkingContentDelivery.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_NETWORKINGCONTENTDELIVERY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/PrivateLink",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_PRIVATELINK,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/PrivateLink.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_PRIVATELINK,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/RTBFabric",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_RTBFABRIC,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/RTBFabric.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_RTBFABRIC,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53HostedZone",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53HOSTEDZONE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53HostedZone.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53HOSTEDZONE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53ReadinessChecks",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53READINESSCHECKS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53ReadinessChecks.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53READINESSCHECKS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53Resolver",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53Resolver.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53ResolverDNSFirewall",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVERDNSFIREWALL,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53ResolverDNSFirewall.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVERDNSFIREWALL,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53ResolverQueryLogging",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVERQUERYLOGGING,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53ResolverQueryLogging.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVERQUERYLOGGING,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53RouteTable",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53ROUTETABLE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53RouteTable.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53ROUTETABLE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53RoutingControls",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53ROUTINGCONTROLS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/Route53RoutingControls.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53ROUTINGCONTROLS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/SitetoSiteVPN",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_SITETOSITEVPN,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/SitetoSiteVPN.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_SITETOSITEVPN,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/TransitGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_TRANSITGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/TransitGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_TRANSITGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCCarrierGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCCARRIERGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCCarrierGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCCARRIERGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCCustomerGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCCUSTOMERGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCCustomerGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCCUSTOMERGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCElasticNetworkInterface",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCELASTICNETWORKINTERFACE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCElasticNetworkInterface.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCELASTICNETWORKINTERFACE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCEndpoints",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCENDPOINTS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCEndpoints.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCENDPOINTS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCInternetGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCINTERNETGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCInternetGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCINTERNETGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCLattice",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCLATTICE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCLattice.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCLATTICE,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCNATGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCNATGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCNATGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCNATGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCNetworkAccessControlList",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCNETWORKACCESSCONTROLLIST,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCNetworkAccessControlList.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCNETWORKACCESSCONTROLLIST,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCPeeringConnection",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCPEERINGCONNECTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCPeeringConnection.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCPEERINGCONNECTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCReachabilityAnalyzer",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCREACHABILITYANALYZER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCReachabilityAnalyzer.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCREACHABILITYANALYZER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCRouter",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCROUTER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCRouter.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCROUTER,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCTrafficMirroring",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCTRAFFICMIRRORING,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCTrafficMirroring.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCTRAFFICMIRRORING,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCVPNConnection",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVPNCONNECTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCVPNConnection.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVPNCONNECTION,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCVPNGateway",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVPNGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCVPNGateway.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVPNGATEWAY,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCVirtualprivatecloudVPC",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVIRTUALPRIVATECLOUDVPC,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VPCVirtualprivatecloudVPC.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVIRTUALPRIVATECLOUDVPC,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VerifiedAccess",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VERIFIEDACCESS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VerifiedAccess.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VERIFIEDACCESS,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VirtualPrivateCloud",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VIRTUALPRIVATECLOUD,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/VirtualPrivateCloud.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_VIRTUALPRIVATECLOUD,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/all",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ALL,
    );
    registry.insert(
        "aws/NetworkingContentDelivery/all.puml",
        AWS_AWS_NETWORKINGCONTENTDELIVERY_ALL,
    );
    registry.insert("aws/Storage/Backup", AWS_AWS_STORAGE_BACKUP);
    registry.insert("aws/Storage/Backup.puml", AWS_AWS_STORAGE_BACKUP);
    registry.insert(
        "aws/Storage/BackupAWSBackupsupportforAmazonFSxforNetAppONTAP",
        AWS_AWS_STORAGE_BACKUPAWSBACKUPSUPPORTFORAMAZONFSXFORNETAPPONTAP,
    );
    registry.insert(
        "aws/Storage/BackupAWSBackupsupportforAmazonFSxforNetAppONTAP.puml",
        AWS_AWS_STORAGE_BACKUPAWSBACKUPSUPPORTFORAMAZONFSXFORNETAPPONTAP,
    );
    registry.insert(
        "aws/Storage/BackupAWSBackupsupportforAmazonS3",
        AWS_AWS_STORAGE_BACKUPAWSBACKUPSUPPORTFORAMAZONS3,
    );
    registry.insert(
        "aws/Storage/BackupAWSBackupsupportforAmazonS3.puml",
        AWS_AWS_STORAGE_BACKUPAWSBACKUPSUPPORTFORAMAZONS3,
    );
    registry.insert(
        "aws/Storage/BackupAuditManager",
        AWS_AWS_STORAGE_BACKUPAUDITMANAGER,
    );
    registry.insert(
        "aws/Storage/BackupAuditManager.puml",
        AWS_AWS_STORAGE_BACKUPAUDITMANAGER,
    );
    registry.insert(
        "aws/Storage/BackupBackupPlan",
        AWS_AWS_STORAGE_BACKUPBACKUPPLAN,
    );
    registry.insert(
        "aws/Storage/BackupBackupPlan.puml",
        AWS_AWS_STORAGE_BACKUPBACKUPPLAN,
    );
    registry.insert(
        "aws/Storage/BackupBackupRestore",
        AWS_AWS_STORAGE_BACKUPBACKUPRESTORE,
    );
    registry.insert(
        "aws/Storage/BackupBackupRestore.puml",
        AWS_AWS_STORAGE_BACKUPBACKUPRESTORE,
    );
    registry.insert(
        "aws/Storage/BackupBackupVault",
        AWS_AWS_STORAGE_BACKUPBACKUPVAULT,
    );
    registry.insert(
        "aws/Storage/BackupBackupVault.puml",
        AWS_AWS_STORAGE_BACKUPBACKUPVAULT,
    );
    registry.insert(
        "aws/Storage/BackupComplianceReporting",
        AWS_AWS_STORAGE_BACKUPCOMPLIANCEREPORTING,
    );
    registry.insert(
        "aws/Storage/BackupComplianceReporting.puml",
        AWS_AWS_STORAGE_BACKUPCOMPLIANCEREPORTING,
    );
    registry.insert("aws/Storage/BackupCompute", AWS_AWS_STORAGE_BACKUPCOMPUTE);
    registry.insert(
        "aws/Storage/BackupCompute.puml",
        AWS_AWS_STORAGE_BACKUPCOMPUTE,
    );
    registry.insert("aws/Storage/BackupGateway", AWS_AWS_STORAGE_BACKUPGATEWAY);
    registry.insert(
        "aws/Storage/BackupGateway.puml",
        AWS_AWS_STORAGE_BACKUPGATEWAY,
    );
    registry.insert(
        "aws/Storage/BackupLegalHold",
        AWS_AWS_STORAGE_BACKUPLEGALHOLD,
    );
    registry.insert(
        "aws/Storage/BackupLegalHold.puml",
        AWS_AWS_STORAGE_BACKUPLEGALHOLD,
    );
    registry.insert(
        "aws/Storage/BackupRecoveryPointObjective",
        AWS_AWS_STORAGE_BACKUPRECOVERYPOINTOBJECTIVE,
    );
    registry.insert(
        "aws/Storage/BackupRecoveryPointObjective.puml",
        AWS_AWS_STORAGE_BACKUPRECOVERYPOINTOBJECTIVE,
    );
    registry.insert(
        "aws/Storage/BackupRecoveryTimeObjective",
        AWS_AWS_STORAGE_BACKUPRECOVERYTIMEOBJECTIVE,
    );
    registry.insert(
        "aws/Storage/BackupRecoveryTimeObjective.puml",
        AWS_AWS_STORAGE_BACKUPRECOVERYTIMEOBJECTIVE,
    );
    registry.insert(
        "aws/Storage/BackupVaultLock",
        AWS_AWS_STORAGE_BACKUPVAULTLOCK,
    );
    registry.insert(
        "aws/Storage/BackupVaultLock.puml",
        AWS_AWS_STORAGE_BACKUPVAULTLOCK,
    );
    registry.insert(
        "aws/Storage/BackupVirtualMachine",
        AWS_AWS_STORAGE_BACKUPVIRTUALMACHINE,
    );
    registry.insert(
        "aws/Storage/BackupVirtualMachine.puml",
        AWS_AWS_STORAGE_BACKUPVIRTUALMACHINE,
    );
    registry.insert(
        "aws/Storage/BackupVirtualMachineMonitor",
        AWS_AWS_STORAGE_BACKUPVIRTUALMACHINEMONITOR,
    );
    registry.insert(
        "aws/Storage/BackupVirtualMachineMonitor.puml",
        AWS_AWS_STORAGE_BACKUPVIRTUALMACHINEMONITOR,
    );
    registry.insert("aws/Storage/EFS", AWS_AWS_STORAGE_EFS);
    registry.insert("aws/Storage/EFS.puml", AWS_AWS_STORAGE_EFS);
    registry.insert(
        "aws/Storage/ElasticBlockStore",
        AWS_AWS_STORAGE_ELASTICBLOCKSTORE,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStore.puml",
        AWS_AWS_STORAGE_ELASTICBLOCKSTORE,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreAmazonDataLifecycleManager",
        AWS_AWS_STORAGE_ELASTICBLOCKSTOREAMAZONDATALIFECYCLEMANAGER,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreAmazonDataLifecycleManager.puml",
        AWS_AWS_STORAGE_ELASTICBLOCKSTOREAMAZONDATALIFECYCLEMANAGER,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreMultipleVolumes",
        AWS_AWS_STORAGE_ELASTICBLOCKSTOREMULTIPLEVOLUMES,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreMultipleVolumes.puml",
        AWS_AWS_STORAGE_ELASTICBLOCKSTOREMULTIPLEVOLUMES,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreSnapshot",
        AWS_AWS_STORAGE_ELASTICBLOCKSTORESNAPSHOT,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreSnapshot.puml",
        AWS_AWS_STORAGE_ELASTICBLOCKSTORESNAPSHOT,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreVolumegp3",
        AWS_AWS_STORAGE_ELASTICBLOCKSTOREVOLUMEGP3,
    );
    registry.insert(
        "aws/Storage/ElasticBlockStoreVolumegp3.puml",
        AWS_AWS_STORAGE_ELASTICBLOCKSTOREVOLUMEGP3,
    );
    registry.insert(
        "aws/Storage/ElasticDisasterRecovery",
        AWS_AWS_STORAGE_ELASTICDISASTERRECOVERY,
    );
    registry.insert(
        "aws/Storage/ElasticDisasterRecovery.puml",
        AWS_AWS_STORAGE_ELASTICDISASTERRECOVERY,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemElasticThroughput",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMELASTICTHROUGHPUT,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemElasticThroughput.puml",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMELASTICTHROUGHPUT,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemFileSystem",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMFILESYSTEM,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemFileSystem.puml",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMFILESYSTEM,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemIntelligentTiering",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMINTELLIGENTTIERING,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemIntelligentTiering.puml",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMINTELLIGENTTIERING,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemOneZone",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMONEZONE,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemOneZone.puml",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMONEZONE,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemOneZoneInfrequentAccess",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMONEZONEINFREQUENTACCESS,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemOneZoneInfrequentAccess.puml",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMONEZONEINFREQUENTACCESS,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemStandard",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMSTANDARD,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemStandard.puml",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMSTANDARD,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemStandardInfrequentAccess",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMSTANDARDINFREQUENTACCESS,
    );
    registry.insert(
        "aws/Storage/ElasticFileSystemStandardInfrequentAccess.puml",
        AWS_AWS_STORAGE_ELASTICFILESYSTEMSTANDARDINFREQUENTACCESS,
    );
    registry.insert("aws/Storage/FSx", AWS_AWS_STORAGE_FSX);
    registry.insert("aws/Storage/FSx.puml", AWS_AWS_STORAGE_FSX);
    registry.insert("aws/Storage/FSxforLustre", AWS_AWS_STORAGE_FSXFORLUSTRE);
    registry.insert(
        "aws/Storage/FSxforLustre.puml",
        AWS_AWS_STORAGE_FSXFORLUSTRE,
    );
    registry.insert(
        "aws/Storage/FSxforNetAppONTAP",
        AWS_AWS_STORAGE_FSXFORNETAPPONTAP,
    );
    registry.insert(
        "aws/Storage/FSxforNetAppONTAP.puml",
        AWS_AWS_STORAGE_FSXFORNETAPPONTAP,
    );
    registry.insert("aws/Storage/FSxforOpenZFS", AWS_AWS_STORAGE_FSXFOROPENZFS);
    registry.insert(
        "aws/Storage/FSxforOpenZFS.puml",
        AWS_AWS_STORAGE_FSXFOROPENZFS,
    );
    registry.insert("aws/Storage/FSxforWFS", AWS_AWS_STORAGE_FSXFORWFS);
    registry.insert("aws/Storage/FSxforWFS.puml", AWS_AWS_STORAGE_FSXFORWFS);
    registry.insert("aws/Storage/FileCache", AWS_AWS_STORAGE_FILECACHE);
    registry.insert("aws/Storage/FileCache.puml", AWS_AWS_STORAGE_FILECACHE);
    registry.insert(
        "aws/Storage/FileCacheHybridNFSlinkeddatasets",
        AWS_AWS_STORAGE_FILECACHEHYBRIDNFSLINKEDDATASETS,
    );
    registry.insert(
        "aws/Storage/FileCacheHybridNFSlinkeddatasets.puml",
        AWS_AWS_STORAGE_FILECACHEHYBRIDNFSLINKEDDATASETS,
    );
    registry.insert(
        "aws/Storage/FileCacheOnpremisesNFSlinkeddatasets",
        AWS_AWS_STORAGE_FILECACHEONPREMISESNFSLINKEDDATASETS,
    );
    registry.insert(
        "aws/Storage/FileCacheOnpremisesNFSlinkeddatasets.puml",
        AWS_AWS_STORAGE_FILECACHEONPREMISESNFSLINKEDDATASETS,
    );
    registry.insert(
        "aws/Storage/FileCacheS3linkeddatasets",
        AWS_AWS_STORAGE_FILECACHES3LINKEDDATASETS,
    );
    registry.insert(
        "aws/Storage/FileCacheS3linkeddatasets.puml",
        AWS_AWS_STORAGE_FILECACHES3LINKEDDATASETS,
    );
    registry.insert("aws/Storage/S3onOutposts", AWS_AWS_STORAGE_S3ONOUTPOSTS);
    registry.insert(
        "aws/Storage/S3onOutposts.puml",
        AWS_AWS_STORAGE_S3ONOUTPOSTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageService",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICE,
    );
    registry.insert(
        "aws/Storage/SimpleStorageService.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICE,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceBucket",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEBUCKET,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceBucket.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEBUCKET,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGeneralAccessPoints",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGENERALACCESSPOINTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGeneralAccessPoints.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGENERALACCESSPOINTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGlacier",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIER,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGlacier.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIER,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGlacierArchive",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIERARCHIVE,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGlacierArchive.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIERARCHIVE,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGlacierVault",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIERVAULT,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceGlacierVault.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIERVAULT,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceObject",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEOBJECT,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceObject.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEOBJECT,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3BatchOperations",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3BATCHOPERATIONS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3BatchOperations.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3BATCHOPERATIONS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Files",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3FILES,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Files.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3FILES,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3GlacierDeepArchive",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3GLACIERDEEPARCHIVE,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3GlacierDeepArchive.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3GLACIERDEEPARCHIVE,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3GlacierFlexibleRetrieval",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3GLACIERFLEXIBLERETRIEVAL,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3GlacierFlexibleRetrieval.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3GLACIERFLEXIBLERETRIEVAL,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3IntelligentTiering",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3INTELLIGENTTIERING,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3IntelligentTiering.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3INTELLIGENTTIERING,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3MultiRegionAccessPoints",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3MULTIREGIONACCESSPOINTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3MultiRegionAccessPoints.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3MULTIREGIONACCESSPOINTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3ObjectLambda",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3OBJECTLAMBDA,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3ObjectLambda.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3OBJECTLAMBDA,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3ObjectLambdaAccessPoints",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3OBJECTLAMBDAACCESSPOINTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3ObjectLambdaAccessPoints.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3OBJECTLAMBDAACCESSPOINTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3OnOutposts",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3ONOUTPOSTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3OnOutposts.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3ONOUTPOSTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3OneZoneIA",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3ONEZONEIA,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3OneZoneIA.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3ONEZONEIA,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Replication",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3REPLICATION,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Replication.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3REPLICATION,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3ReplicationTimeControl",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3REPLICATIONTIMECONTROL,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3ReplicationTimeControl.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3REPLICATIONTIMECONTROL,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Standard",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3STANDARD,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Standard.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3STANDARD,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3StandardIA",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3STANDARDIA,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3StandardIA.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3STANDARDIA,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Tables",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3TABLES,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Tables.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3TABLES,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Vectors",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3VECTORS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceS3Vectors.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3VECTORS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceVPCAccessPoints",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEVPCACCESSPOINTS,
    );
    registry.insert(
        "aws/Storage/SimpleStorageServiceVPCAccessPoints.puml",
        AWS_AWS_STORAGE_SIMPLESTORAGESERVICEVPCACCESSPOINTS,
    );
    registry.insert("aws/Storage/Snowball", AWS_AWS_STORAGE_SNOWBALL);
    registry.insert("aws/Storage/Snowball.puml", AWS_AWS_STORAGE_SNOWBALL);
    registry.insert("aws/Storage/SnowballEdge", AWS_AWS_STORAGE_SNOWBALLEDGE);
    registry.insert(
        "aws/Storage/SnowballEdge.puml",
        AWS_AWS_STORAGE_SNOWBALLEDGE,
    );
    registry.insert(
        "aws/Storage/SnowballSnowballImportExport",
        AWS_AWS_STORAGE_SNOWBALLSNOWBALLIMPORTEXPORT,
    );
    registry.insert(
        "aws/Storage/SnowballSnowballImportExport.puml",
        AWS_AWS_STORAGE_SNOWBALLSNOWBALLIMPORTEXPORT,
    );
    registry.insert("aws/Storage/Storage", AWS_AWS_STORAGE_STORAGE);
    registry.insert("aws/Storage/Storage.puml", AWS_AWS_STORAGE_STORAGE);
    registry.insert("aws/Storage/StorageGateway", AWS_AWS_STORAGE_STORAGEGATEWAY);
    registry.insert(
        "aws/Storage/StorageGateway.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayAmazonFSxFileGateway",
        AWS_AWS_STORAGE_STORAGEGATEWAYAMAZONFSXFILEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayAmazonFSxFileGateway.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYAMAZONFSXFILEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayAmazonS3FileGateway",
        AWS_AWS_STORAGE_STORAGEGATEWAYAMAZONS3FILEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayAmazonS3FileGateway.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYAMAZONS3FILEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayCachedVolume",
        AWS_AWS_STORAGE_STORAGEGATEWAYCACHEDVOLUME,
    );
    registry.insert(
        "aws/Storage/StorageGatewayCachedVolume.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYCACHEDVOLUME,
    );
    registry.insert(
        "aws/Storage/StorageGatewayFileGateway",
        AWS_AWS_STORAGE_STORAGEGATEWAYFILEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayFileGateway.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYFILEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayNoncachedVolume",
        AWS_AWS_STORAGE_STORAGEGATEWAYNONCACHEDVOLUME,
    );
    registry.insert(
        "aws/Storage/StorageGatewayNoncachedVolume.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYNONCACHEDVOLUME,
    );
    registry.insert(
        "aws/Storage/StorageGatewayTapeGateway",
        AWS_AWS_STORAGE_STORAGEGATEWAYTAPEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayTapeGateway.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYTAPEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayVirtualTapeLibrary",
        AWS_AWS_STORAGE_STORAGEGATEWAYVIRTUALTAPELIBRARY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayVirtualTapeLibrary.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYVIRTUALTAPELIBRARY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayVolumeGateway",
        AWS_AWS_STORAGE_STORAGEGATEWAYVOLUMEGATEWAY,
    );
    registry.insert(
        "aws/Storage/StorageGatewayVolumeGateway.puml",
        AWS_AWS_STORAGE_STORAGEGATEWAYVOLUMEGATEWAY,
    );
    registry.insert("aws/Storage/all", AWS_AWS_STORAGE_ALL);
    registry.insert("aws/Storage/all.puml", AWS_AWS_STORAGE_ALL);
}

/// `aws/Compute/AppRunner`
const AWS_AWS_COMPUTE_APPRUNNER: &str = r#"sprite $AppRunner [64x64/16z] {
xTPL0eCm54JHYUz_nRMXWfD9dNU_i8DR-Pai2ymtUXIzxEXrxe0cdXUyHo_VcvB8ouB784xfysNV7-jD8PSzEHCyUP8zEMGVUEeqpxxVc-ytCOKfpo-dV4_w
_kldx_ll7_CvDldi46NVVQHKt-E2xxZYWHwNF7cIdX_WohNd5o1N0yXrJ_wfVqFVyx-GPHq1
}"#;

/// `aws/Compute/Batch`
const AWS_AWS_COMPUTE_BATCH: &str = r#"sprite $Batch [64x64/16z] {
xLG5SWGn39Fn_l_ZTY6UgxUPm37eCA2COqbxUbPyyOKB0IG-ILViq5awcxlnt04UD0Lyd2_WyfSXuto2bRy66kS3vrUvb7zf_PVsxrBzBlVFvVwzF3yNgFWI
V660T9RlX0qU2j3Vyum0jhfZQT7xzoSik23CbyOPKdW4BVghkuR-CVw_0eQTNAJ49SxHSYwNCLTQxKk5iZZ_l16Em_WcV6hKdySPUc7_JhXCl_0FLVspZ3d6
HoYr_0kqy_8M_7O_0hc_TJsCA_bD-YO_e4z_2tyT_9ZNCxA_8m_CNir3HihNCkGzZsczcoy3_FnyuUMUeikz-RZUxf-Cz2_hqtuUPsH_jFtJXd3uyGdnnHS_
}"#;

/// `aws/Compute/Bottlerocket`
const AWS_AWS_COMPUTE_BOTTLEROCKET: &str = r#"sprite $Bottlerocket [64x64/16z] {
xPG5jcGn20KB2B3_5OzaQBUGiM_LyhHYzuHyitpnXOFdFW4GlwEhij-08FMtXIAH6O9kwLOZ3x0DVO0VGnXjlQHJKnsk1sqGNLtSL014pANPW723j983n3mY
Cy9DWKwAe5ShxWhhViJTUmr4RgvRmFNrF_L7lHzcKVvewCxqGm4qfe_toawc0Qg0JPzer0sOZaOea0c2D3P--G3b1zXwpIr7GQkjnLfSImstlm5hVlr196HG
VhPz38nTFvI9nfwVwGBYcJj-p8invY6Qle9VPQeDtu1njP-mPTz0aSWZ8IZOgW-U0b9360AI3ik-LtWvgGXzsVUpDJ_Bmg5LV4z-lmHV-yK_zTrFnmtaZ1qV
JkmD7zn__USM83-0t8HYrm-VHFubllZYEm
}"#;

/// `aws/Compute/ComputeOptimizer2`
const AWS_AWS_COMPUTE_COMPUTEOPTIMIZER2: &str = r#"sprite $ComputeOptimizer2 [64x64/16z] {
xPQ7bkCm24KlrV__nLjO6rbJ56LxOoO9oj7runsCukFtY__nFqB4lm4t02nh0SIJuc2XW4Rns9FY9PW6I7tnnKlkYiEvyCgKgaEnBN69ve_A712t7HzIUoQ-
d0Y0FRv0EP_vjkM7hoxG-r_p7GwsoNSsXOC_v2k-dL_Nly_dlRkon0FYoffdQ2Ra-tGgW6PboE2HYx95_GwO0Uw03nSXBPlm7Yv3khB1rmJqhlV6ULZlhFc4
64M-yj1Rm-vDiEpKqT64z_XKYC4xE7HDZtd0xqrIQ6oBropPxfStdpjydA8gICwO8u_v7f0KP2vRy3xOMV67PUkEpo4l0BxWxuiu_LktFo4YbgENDemMhwqu
tGUlUSlfX52mppn8Zas-kF_HK4w558Wgofi-OZN_eT2b1gKAbnUf-YAEHQH1NjPrdXzwy5O1RVs-5YHgijrpW_TMWhRzUlZNDz5OZnzWK6I3dzVHefbbvHc-
Mt66Bor0T2ykNiQyZCDmVOYMg9yyjFho3ExolFl_F5zTbgl_3g_gpLwlde3yCoqWPMMAhcTjecZl2uxGnfScYlws1zZVFFx7__W8
}"#;

/// `aws/Compute/DCV`
const AWS_AWS_COMPUTE_DCV: &str = r#"sprite $DCV [64x64/16z] {
xTRLOSGm44NHEzn_nU5eURoom_7vrXDJ_KoxNIWW5YlJl1ElUGPYMHKceBaYFbfraAmf3b532kJasAEEoLmF1AiJAUXKytKc8IQQzpeNzySJQWl7QWiXQWje
DxLeL2UGRcgUUJMyM-gi38Dg6BQyFHgALoV1D-G3jFfyjAFhutq-1AmQRVzJGQ8M67upZQ_T_g60PRLI_Vh-SHdfHe1SsRzXZ7GdtYvgo1hIvCud8C7Epc-O
73VTK-pu0AGhppHgyExeZcu7yfOLYtvfZoc8FX5UgUVKravb4CzRNUo4kqTamBQBMV3bcsd-Xb5yzW4ybitmcw8ptHyCsVI4bg1DbSiIjFcIJ5SGu3KbW3Xg
NMjIojj6JXUmckUyawXLN0LGhvzejti4
}"#;

/// `aws/Compute/EC2AMI`
const AWS_AWS_COMPUTE_EC2AMI: &str = r#"sprite $EC2AMI [64x64/16z] {
xPPLOiOW40PP_Uv_uYhPUC9IxyyyOODR-JKGDqCxMHmfZ5TDEzBOHbVXPiHM5NX4MeWcTHfxKez6AK8b3qS78f33uUz30QJ7Ls3g27-6xvcrt_kC1CU-QM0N
iRfoUvzB802TnVJh_U4FdrS6D_ZjqEUVloBQ3E_y9F_G9xJZNtD_gWHs5VjzvsVumn--M-3RcArYNt3_T_x_-l-lLlq2MEV_EIgYl2xnRMV0qpe1M8p4ETqw
P3aLS4tWgBslADB8UZZgIE1X_tMU0G
}"#;

/// `aws/Compute/EC2AWSMicroserviceExtractorforNET`
const AWS_AWS_COMPUTE_EC2AWSMICROSERVICEEXTRACTORFORNET: &str = r#"sprite $EC2AWSMicroserviceExtractorforNET [64x64/16z] {
pPS7Zkqm24K7JDd_Ydz1ldevagn4q-yqjuCX3nFFs--MsOVexCvtM_3iKVhxwSptMV0QpYQ6Vof_5ps2XSTqs6l4uhc5HPeMrhjeMhYdJzEoS4-_muA1lh5W
XudruFbY6_3mxHZZcS_6mpzy19qdNawa7m_4e6TU3Z8CZf_vwpuLc2pvTSShJZq8TczvkfikTnJGFSzJMCMJTy-p-gaEl8EVo94EHpvDVD9JxtOEf7ZBDwcw
DfzQ3nxz77wWY-ir_3GElBRg67uQKlOKwgMJUUQjPbdcE78ni4MS_CXwA-bNZrnzyXmQ5KUjW9EdN1K8CbOVyjHId90LSFBKAk07cOb_roN0kyINd3Xh9Mh6
JghXbIge9RaOUsyjv4J-vvaggxVAc1CPOtWX9_OtyOf8vDeug5bdKZLmrmxNenA3QxuO5bEbnZufdKID6fEA_M60ST36F-OFeZTinJMiJPGGslu4dlux_FVu
9uV_3l_rK558mZQjpsyp5YWRtIMhdSokTBLVMUd93Crfoe4qtI7qebtR4vl16UsSGRrR2ebMgH9ageQEYZLxU6lfu5BoK1WkGxZoMA5-IXPP5SpzLuTd8Aeo
xafANJT8yS_LvFiXXjE1pcCnkvYXp-MN0FXMlfUWHEkXa5InLYXLtc5Wz5wSTyxt___wk_KN
}"#;

/// `aws/Compute/EC2AutoScaling`
const AWS_AWS_COMPUTE_EC2AUTOSCALING: &str = r#"sprite $EC2AutoScaling [64x64/16z] {
xPM7Ji0m30KlyVh_FqOXL6PqcWqzgRltf4p7t-Ncfaj47M_GwxenQ5xK5Heqi5hJGHp6LIk-e37y42mgU4m_FAfCFtwj7whnFNYNvIYj0gU5mt9KvhxqFFoa
vlCimakQoFB37eO-3Hke9W0nK6pF5z3Xgz0YSL2TEX8d0i9V-v7wSO1BV-sB7Eao0TJXjGz3HoPhFibw-qdMV9q0jjv-oq7PtRLig_sMk_6k7kljFn4WO7lZ
RmS3Q4DV7__yuPZrzf_7OHW9m1LVzCtwq-lhN_Buylt7HH-FM8uQejzNFtpng_NVfgzepSU6hw1H3518lH80gHS3DEhqr6k8ni_bvkO1
}"#;

/// `aws/Compute/EC2DBInstance`
const AWS_AWS_COMPUTE_EC2DBINSTANCE: &str = r#"sprite $EC2DBInstance [64x64/16z] {
xPS5SWD130Ip8vZ__pYbCXxff82v3OUzZ5-Fn03Y08p3ONalHyaua1e6YSczZv-h-oI1iCYsxxgB9twZVFxM7nfKVGqcJHaCYlu1_DQ_R_dTtxzt-V-s_RnK
_I6e-WZkLNyoyPlf3t2WNtvJxkH3p9rqDvNHRh_YlCRHAtyxM6PgvWoaumlfLuc6aXoD_1ivvW6YvpkkEpsxVc2g8gBcPDjVWdhvXJ_4d0TC1zh-OjX5qdxg
_e7x-wuReEGdL7pzfP-l7tF_u-VxbyGlSdzVq_5l7v2EQNZUq4EuZE13VRcABKx-xZwPV-eCvHcNfdm0QcQJsVFUV0C
}"#;

/// `aws/Compute/EC2ElasticIPAddress`
const AWS_AWS_COMPUTE_EC2ELASTICIPADDRESS: &str = r#"sprite $EC2ElasticIPAddress [64x64/16z] {
xT85RiH0341Hyx__YOkJWYjP0N7d2NUJcENVsxPjqyYYAapkiE9GDo84G6Lc0QHCh5IQ149N6q1DVGBQi_CZfjPSmNhxcqFrSc8AqDwlNb3_fyjV0HoyZwWL
C0P8aAuecIIaB93I1QXClcUiH_cz0Piu4CUh9froXZtzVghVTUA0Z_EJYHR4W_qLn70i3dvYUqlr2Ifm_dCnG5jpVkhubZNCnGhYvF4l0PH_Vfr0pgClwPmF
nGUNYQRANy4XLMwnoCmmEM7Rjcrx1G
}"#;

/// `aws/Compute/EC2ImageBuilder`
const AWS_AWS_COMPUTE_EC2IMAGEBUILDER: &str = r#"sprite $EC2ImageBuilder [64x64/16z] {
xPP7UiGm24OV0g7xdpWz8V8tLXbcgN-9_TIexKD7HqSyLWEm9CyIVsjy3MzIDlYg8lpnvaq1MUHT0GWU10naYLT0jFgViHR0ceFCUHSeVZKMQoqM6F0ErBj7
TIXZta7ytYc6Z7b1UgEfUU1WGrx1UwC2Klzm7TxVeLwCReLVhCpUNocFBgK_2pZua8_lvl_Vh_xSks0ZlYBnKDtx6_ZtEe3TyOh6EtTyAsXa-HqlrC1xliRJ
ZNWF_F8hp7deyPxt9JvYV9yNVB3_olrhTu7TzrCqtd_VVx69WJqHF_581hlns-UF-I1_qlcRhn_v-fMld_dwdUyV-VwLxv_v_fsV7yPyUdvw_VnsT7Jq3W
}"#;

/// `aws/Compute/EC2Instance`
const AWS_AWS_COMPUTE_EC2INSTANCE: &str = r#"sprite $EC2Instance [64x64/16z] {
xPS34i0m4023m_zVN0zhQxV8A3fVU1OfBHWdfhkd9UFnyval5UgkIyQXwRu2EML-MCZ1yrE3hfclxPnEPnqhvdUalVDtdlyc-QD_z8_-qJ_wH__e7_tVK7_k
hf_lh_-Plxz_lPyI
}"#;

/// `aws/Compute/EC2Instances`
const AWS_AWS_COMPUTE_EC2INSTANCES: &str = r#"sprite $EC2Instances [64x64/16z] {
xTP7jWGX34DHB5Eb_Q_uXyv9eCw9D-NS6M2NLmWoHoSReU_PhCj1LeWg6OQ7SVQ5VlZ83CiRcJuWStobRewk1rlDNPyaGZIHZeyYgb_jXqVcHTwV7tlUVexS
X3tlpaPe7zviZ5Nbq8FC49sSZSSUHPHtyCCFdrWNfbSF1hxVb9VwJR6fM5xtT3_yy3L5a-73MoTrsU8eHY4hhz0l
}"#;

/// `aws/Compute/EC2InstancewithCloudWatch`
const AWS_AWS_COMPUTE_EC2INSTANCEWITHCLOUDWATCH: &str = r#"sprite $EC2InstancewithCloudWatch [64x64/16z] {
tPO7rjmW30QjWdJ_4wVWp5iJbXWv_VzUsGA3ElRnpqXqMpB15bbIqy4qT47jEznDjsLjE23uiITCEh5NSOmAt_AeInC7-59s8CvGCkq9xvaueX-FLtDtqne_
ZqDOZP_78QJAwrVAlsBI_T0QBtbjAEU0RJx6VhBChF26zSk-9jwQxV7pD3LIgRSyOpNbCtj9v9vtTesJfcSSztnxaoqn-ROa6tpStXXhNbgk-IEJqYvuus9-
hqO_BtZbmJ1AlRdBGJ-7h7ZS6-JCaV8p5ho1BqRH2K8ddZ9FE8R5oOnmqCIpC3Qp7-hkqZqnpEpmpDBP_cSeTDW6Zodlk96V7_9jYjwb5qpOiFIVWn4hnYgz
lSwViN89NuoXmS2oVkFDnGf3W_r6b6zuwVqhw7Gr-dxY5zfluXc0byvKwRcV_7kJc_bDH4OkhHotzuUe8VhlXNF0ah_fVswNJHwhI0uEsEFPXe83zddk7-ie
1vHuz3h0d_0Si9YVxGFiES-Zr1xnG7HKaKSwzFzFlhyobi_Vdtakb_MQJQMbTiLaNSkgPUjfGrd32ugStYMgiZ4SZmeSqF-zFWC
}"#;

/// `aws/Compute/EC2Rescue`
const AWS_AWS_COMPUTE_EC2RESCUE: &str = r#"sprite $EC2Rescue [64x64/16z] {
rPQ7aiGW28O32hp_4rzXPdyBNiJh-qtPudn9K21S_mRYD1JaedQ4S6yJx3mowRLmcbBDJE4hhfP7cbCETLo0SVjh0HL0gsNSrQokV6le1m_mk50vyn6sFnSZ
430-bdpAvyutElMBMk-RbgGFU_JDb5C-XDhx5HcxyP7Dy_vvnjBMzvyc5FUV51UuymbnJxx7eRJp5ThiUmowyGLMyB6wycCf3ljVsjFnsiSwjKFGulXNx8uk
VVnLeO-qBU2LZzlZYuY1Ycw3gb_vzBXz3Sc6FH2tea_M4toTeg7PBpOnDQi9NlXIWKPVAl1WeayNgD4lqwydy9_CbuP6NuVLyjtvG_g6cJheOK2ihQcgtDGF
sWW1pt_KJx9-rO3eldxHFvpOFn07x_eN0XPsj9KpuzliOrsnlEwVjElVakpVv-yFkNz_eFDAzA5lVBmdWy_-V_hzJQDFde39-Q5wpZJVV-3dthzuCRy8dSnF
BAClV3g_KUz_RduKSZRpurVcrxlvMVVpysryn62v78eoIIn5mUccGPaWL2a7pDaiTeo4VCkpZ9VoNF-7Lm
}"#;

/// `aws/Compute/EC2SpotInstance`
const AWS_AWS_COMPUTE_EC2SPOTINSTANCE: &str = r#"sprite $EC2SpotInstance [64x64/16z] {
tPS7Tjj044Cr5VU_SMZn1U4il7ugwN1b-PZ6ihhzrJBtjtW7tZA8rtdFQjpLbM8YccT4OL7vSlfKNVDCdEege9X6NElAjKyqY3DUs8_-sPaFudAAVMxWVGtb
LVQTkH-m4sSqCKYWVJiCn8AhGR4oLPRWOl2b11U3n_773VIKrmrgTuBdPmOMGnzxZfyqvHlrYG6ccgTgNv1QWcEHdpKWLlxkBGOEX56ycm4iU1AYWJu_k1L0
IFYHjEruFLqD7SdsXodFTAcQ1NqmFhJm1kHClsISGxLKqwC0btx8u6pooHPnQoXaSEGPSPOll3HkJ8CDa6eUunibxNk8rv0UGBbnyv0fR-v9thq1JlMkucGF
fV85W7oTZ2aVN_9zK_vc3fJP69poTeYxp0ema_wnSShBzTcoyI2l8olWUJxdzIjyEfN2En2oHNvp_r2CADKmyS9GRTed3M3XDhGiEGn8BOSyPPcsMQ6uvBEz
SVIz0BG-BtQy6gI-G03GI0St3OpXzVqLEZWng5dzVA2ql375e50taSKv8ttmgq7ORd5IRom_T5duk84kZHutKFnn0yMxjgXLBGQA9vV9Kpmq33HvAnnAMzVF
bZXK9YL8KT4urEcN5q-U-tnTeL19d0v0TmFLEDNB2cLgs9q1AQQZf-zbaMVehWpx0v__oF_N-WO
}"#;

/// `aws/Compute/ElasticBeanstalk`
const AWS_AWS_COMPUTE_ELASTICBEANSTALK: &str = r#"sprite $ElasticBeanstalk [64x64/16z] {
xPC9ZeGm30JBDZR_V_6UaLg3XRZsNahd09rKsfdvft5pSvDXW4MTIniCB8_70uYiLmjWEXgV8BGM-47vnorJg-rZQPoGyrorszvQ7HQmLmEoFm8CRDVfhIj1
xYrYGdMNWzVhmnf5RET3cnHlg3WZD_LDxJjUvSZ7iOtBvA0zeDuajFZ65CeWQcvvCJdoQHYcTFFVl8wfhbf_eiQJZe-L-_o49jIXBmoRDqn0sxZdrMW3si5X
zRkEaSlYeRTGQ_w1BmSFakd_a7wmR7d9BB_g16JpNzp2Ckat77mgARTyXewoOW2eiaV_WF5ybGe3pUYnlp2Fd7V1GFCxXZCO5YVvzlpWOjul-8j_bT_cvkRc
6G
}"#;

/// `aws/Compute/ElasticFabricAdapter`
const AWS_AWS_COMPUTE_ELASTICFABRICADAPTER: &str = r#"sprite $ElasticFabricAdapter [64x64/16z] {
xPO5WeKm30NdXJJS_yJh4go1TUby1J9rctyWWy4Wt0GWyxXkkqZ8hzbDW7cxopYQ6w1sGRTrZYwmqnKN-AO-e3YdxqU68CxfTbYhg7sXtgCAxsGIu7NfMwTR
onfCMBzZfshOtMVFBW_ZgXw87RyUiuyZTIyMSiGEuWaz-Eky8KVi0C-_1twAEEM_tloLVkBZ_Rh_ra2AACSFUIG-_wI8G-B1slMV7_UOQAi9CEV1tyhvXtg-
oWM4NeJ3kzUl2aDne0kRQuJYG5VbFiT5SRjAW6dRiOLUxv-Vldzdt4YOl_FyyWCP30Qt
}"#;

/// `aws/Compute/ElasticVMwareService`
const AWS_AWS_COMPUTE_ELASTICVMWARESERVICE: &str = r#"sprite $ElasticVMwareService [64x64/16z] {
xPG7bWGX20LBnC_zJxnvVMq7T7EioQ5a-EZ4z-EVVrGBG76zoYvqJ2_M7R2c23K3_8LwtHQLGNrfTKzUpz2XdeFMVSEERr6W-69vFKv0s_SUqKhtTc-Ct_RI
56rfWKBh1NMHJ0ReMJzqqYWB_cFW8F2n4y5CRt3Vhm92lky4hENxldyNZibhCWG1LT7ziPDxvCa20jCsXw6JVh0l5nZ60vjd0vLo5PssVjBbQOY6nueV5OiJ
2Zhu34Iw3I3M_7Y97vsvhpLVvmEmv7GDlcF9_18_dryKB6AUMeNhVqxDV5bw071bgQjE3o2P3nH5GhEI-jPYHk8dP3wFV9NV-MmVXkUVw0Fub_bVFmSutd_X
B_-c_FFFBG
}"#;

/// `aws/Compute/Lambda`
const AWS_AWS_COMPUTE_LAMBDA: &str = r#"sprite $Lambda [64x64/16z] {
xPC5TiLG509vclslk7PhKv-wyLq8Sn8ulfN-zQyHQ5b_2zIyVz_wH03l7-blyk_We_8JmFi9uFsbdk7z193-KYE0xzyyIgi0f6RQ3T6KPZPlfhqLO7qs09iN
3S3c-H8Fk6o9PxBV0f2aetaEOAO1m9OuMVL_09ZuYgxgMu3gU_s_cHNXdvS0M9681u0fSSNNgrqHELk6M17Y0U1Mr3D8OahCeYRBhOZnCqtEc165hrPK2JoH
ND6vkeedm8L4FDwMacVY6LeeyBjuVg2VIXRY0V2CdmNdbMBr4nlFwYSsdW2F-LjA3tOaj_GI_xuBXsD_3_thNwS
}"#;

/// `aws/Compute/LambdaLambdaFunction`
const AWS_AWS_COMPUTE_LAMBDALAMBDAFUNCTION: &str = r#"sprite $LambdaLambdaFunction [64x64/16z] {
pPU9SXmX30KZ8SZ__t467rtR5KhhswiSdXfetXC83_nvv8Z8ZFWGcZN_4hFoVV049UQRXnZh2LYpHbunQhwy66-YdzM6qexnxEZk2B4GEjfQSLzylBULs2Rh
chwRi5xjr5hyckMbUVUqPpF0UadFwxR-dw1rLdyT-Vg9_Oa1zMlmpHTlGvsR0SQs1Q_ESKYKLK27Vk9bm9qLYppX6mE9MTpVC5PCcbG7pk0on3FhLrWp340C
3KVz70sSYQNDjgCw0qQImGWmQTb0CiKd0J1hUPRzI80T3I3Qs3V6SXyJg4E5pDq8LlHL4TPhttKkC32AsEbDfU-sZGO6NCIyKxrRlZD0Kgzjux33Dmg60KzW
m1EeRGDWBcBpgSb5h31mBc8uyoGRdG4u_5i-NnOT30Dz_iZpyxoBuBr-51i-P62Tri_rGx5JeJRWD5M_bWVmBdBzGeJbUTABOYJSUV_I2Nc_uTSg-rSJ87aI
bu4To-dpYAYUF2DZi0g8Ae4bUHK47MJV5AAiEmRSbPzW5kK90-OXc5b_1lFy4OVV0068A8H6WtSfyFrEdLVTJ6oUxxUF17NBX_Qw1WwQVNw9AYBTF8e8pY_D
gQY9xW3s2PppOnES7xlpQnjvzm1S7Lx2Ftj-__pzuVFtby_Vdpv_V_ly_R6_lpvo_0C
}"#;

/// `aws/Compute/Lightsail`
const AWS_AWS_COMPUTE_LIGHTSAIL: &str = r#"sprite $Lightsail [64x64/16z] {
xLO5ia8n3Dn8zz__n3YXVpAynSUwU4jS_xw3XOK5ffi1PfvydZeD4ooVf0O0pt4Gp94E04zme04svfDXW3rgnG0mkaPrgCbz5XL-LihB1aKUys_Mxz0Qp7_d
pPei1j871hzyypAyo1oc0yUhkSOpJVgL9wKBoiII3v1rce0GR_MTL_FdOzjMe5yxObPhmBQTtqzuydoKWTnItvju_y_Ojmm8v9rm64OJeJ60BB7Q0LsTChiS
m3lqG7i3aDqpmAFq1eIWbydiGl-t_AVZDm3UqeOgVlqou5K5VpOfydp-qk6i_7sgVYu5x39z4jPTM5Qm1EZ-KR3MFsJ_EYCgSZObJpxOFnbmFj4_M_-cSPf6
hN_h-N5rX9WVUdw5cb_lpy_tvxVU7twsl-ZzwNqiB2pi0G
}"#;

/// `aws/Compute/LightsailforResearch`
const AWS_AWS_COMPUTE_LIGHTSAILFORRESEARCH: &str = r#"sprite $LightsailforResearch [64x64/16z] {
xPC5Rkr03037kFS_yOUIvUfjwaHGoeYM9Z9bVJrkRj8L4FLhkl66dRS3KC__KGX8djSbNZSEv3cznvo2dF85Ut-W9tH7LYSXmdJO2y4ttHXLCk7H6UOkaqh6
esePJ_rGR92-3cBR4qV5_j1F5yfF4n3Byi7YC7Z0smwWXb7W83u1ZSg4frqer67Qleq6qOfbR-TzddLNtMnVqpPIktTTA8JOzDIm9fQmwLKY1oDMGZvTFj5z
MAjQaR0V3iflyTV5yHZ1fdvrcoE_-gT8kmrau2y7ZKmNY7Qh-CIl-Ur-G1xx_VznGxflQ-8NWhRwhPD-WhVRiL-LoEj-bVool-9Q_CNNzszV6Q4xdo5NeYkE
_9pxr_ENdPyoqcDz4Mvk_W8
}"#;

/// `aws/Compute/LocalZones`
const AWS_AWS_COMPUTE_LOCALZONES: &str = r#"sprite $LocalZones [64x64/16z] {
xPP7jkn0349HPFyh_a4CVFhLmFBFePcyY0kkh3hDFrDllLNX1i0yYu_L0LoodDybjnftP3MxCW0OIKCk3pV0YvVActIWTyK179NtzWQy1N3uHQaHbXXLbWb8
SswkxG5hnRKOVb5JASdoLVaMmNpvPP3SPF3vJrE8YqWecY_WzS1wjSlNzoD8sCV_QnkWtfitA12al-ISBlf3tXOm50bqjy1bkO0MG1RiTlWz_nL8JG424ZGq
6xZrJqhbS5WOd6mONSbM_j0_GaSEK2LQ-5j-D4z_0mj-1ULd0044MSg_u1U8QQjNpG3D7k8NY1l_d01K0wdvQHMx0ALR00WuyrgW-TOd65Rxqq3D3bpRKz1p
D_jBUIieHUaPg_tLZbpxUwsGTciJG8PZXPPNcCsGFjpVkjYua52TwkNn6i6X3VphfDwcEdKrFQYi1UHnBu9hAUm9Ro-7VKZlVA9-RP3Ayx318lrc1dlsP-KJ
ndC4pyjVq0wK3uVOx6DqZn0SvHyYi4tv_IUvvmoJFqrllVKP
}"#;

/// `aws/Compute/NitroEnclaves`
const AWS_AWS_COMPUTE_NITROENCLAVES: &str = r#"sprite $NitroEnclaves [64x64/16z] {
xTK5We0m34JHtqcRtF_4wugJh2-3LXukyLDovOfuZDwMfaMjUvxpjiIqkEMdYtdnD_ZdnAIuxoTNUbhSzplvwHxGtlEPRmT0Ntn-JILlKF43LF0Ez88NCFA-
0qJQEnWjxnjcMDeRBKJF-W4UFAQTzmuzGcalzEP0JliE_kP0pdg7CJcGKxu1W1uF9ErvF93ayzDHD2ooth3gymkUzm0zqdu0Yhmtm0jUCABWmQBYfUh_up_w
pUnxiH75JygLAxS
}"#;

/// `aws/Compute/Outpostsfamily`
const AWS_AWS_COMPUTE_OUTPOSTSFAMILY: &str = r#"sprite $Outpostsfamily [64x64/16z] {
xPK50fr034HVTEz_uifMy0rIB_DTNcm2j3zGhrw5crdSX5FvAiqRk0DYxYQ0NSPjIfiAUXsVzHrN0qJ79-N55WJR1CmMxdaolMmR4ZdjXU8RDpki81KRhQNl
JBIFC0Mh9iV-oiNNtndowG9834ORvUQDx3R4gzK17pQ8ZsQFGXOTwY1xEbgkfnq4XsZIwrEsqkblUP1UMz8jOa_p_-sffmxl_DPecEfav6x9juC68C_nGfRv
lxevlTJv3Llmi7pf_W_wJyY2L-ne_lrNoM7zFKDVatq9M1xFhwSFqFAoiSydU74-3vLZtYYElpRsJ-Zz3JJsBye0bVyzmjtwlmv82xRS_x-7Lm3qDWzT3_Zf
_SlNq7azvly8lNhr4G
}"#;

/// `aws/Compute/Outpostsrack`
const AWS_AWS_COMPUTE_OUTPOSTSRACK: &str = r#"sprite $Outpostsrack [64x64/16z] {
xP1LWiKm30D73jt_n0_A_ekRvSsKOLofxOyocHGJMAhtx2esqYqToBNLOlS62DjhG0xh6LtECIasGvJhnHlLK7iNncfHv3SKp5rk8xjrEJzA0V8ovOfAV_cb
gl2eqqzRU7pXxlvG1_tshTyNwH1OQ6O1lq2g0txBl0dbzrGRyrkW_wVve_d_SV_fJt_wmX7sYt3a5WNS5eQ5ynZ50Emy_YmcaoS
}"#;

/// `aws/Compute/Outpostsservers`
const AWS_AWS_COMPUTE_OUTPOSTSSERVERS: &str = r#"sprite $Outpostsservers [64x64/16z] {
xT85Tk0m38JXtyBxdtYthB9jbLj_eKUJw8sItyTn78S9C2oonGSth1M__dJu0CrjKIdlJ32G6WN2pM9l-YowtOEYcGLG0TyOtnzkCZ8LoLKGprKQwwkFvuPY
n4hyXQvS05zR_VD269FqfAU1RNR_a26DxaimvldNo-dCNrJN-kkBsdwF0P7AbBxy_rQyyuAC1Cy9XyuMopGFcG3rB9-Hxy__rlww-olQY3SSnt7y1m
}"#;

/// `aws/Compute/ParallelCluster`
const AWS_AWS_COMPUTE_PARALLELCLUSTER: &str = r#"sprite $ParallelCluster [64x64/16z] {
xPO7SeKX28JNPU7T_yIfVpFXpJ2Q7dUAzGFitZvCIqjBmguOuddmFiPl77yVBm0nmNESFv5vVjCGFxf_qf17UMMkGsGZ5QMbTx3hg8PTLjdw3a2dxmQ0SdSH
OENenQMZhbuzCP3AWVQIiJxckBUoOM_AhmO8gAitiBPy0V1cIAfJ_jWnNGH9j0sHyzaSR7ZFUyN_nglA4qhZL-OeaS1a_lJU0Bj--SAUEqM0QXUxEWC2U6IY
T37mt6dUBda0AbrC1e0c8_hpG-0urJb_kjraqTLxzPO9cPdYti7S-E4tSOpV4Cxp0gRv-FLl-lZwVSkVAjVIqj8Z
}"#;

/// `aws/Compute/ParallelComputingService`
const AWS_AWS_COMPUTE_PARALLELCOMPUTINGSERVICE: &str = r#"sprite $ParallelComputingService [64x64/16z] {
xPG93kim24GVOEFxd_WlDX9HKOEaL7jUrz7K0OzfrgzuUNaP0iXeh6-u47XlLHmOldm0tbeLqEEv8frL6U1PRZHMHHXxd-9h3QHOGLYLNME7lAiKAmYh0lkD
-2nMYiVrd-__UVuL8G-PnkhcRvRUZjKCS68hSGC2yVR_Xooxrfc0-N76kcSY3hPRiIm-bu4BSpKCx4GSBKJnSmBAM0t6s6dDauQTud79WQw6qwA1dmymtylj
R6urH6mA7h7RNgwterk6sq4cqRyhU5wzYVyw2f42UEwkZNypSrGqrokZZpy7RcOd5fVNFluuSpU8w6CMsWEO2kYqA2ri99gmi6-Q9p2_Qlimlo-tS-ZweIrY
lCZA959sc4KNkw1OtFqCBNhBpP4LyBZxEbGDVdy3FZ_YV5HDBeAdzP_k_zVvb_CkkiXApblHHTxEVzLLFkVbvUKl
}"#;

/// `aws/Compute/ServerlessApplicationRepository`
const AWS_AWS_COMPUTE_SERVERLESSAPPLICATIONREPOSITORY: &str = r#"sprite $ServerlessApplicationRepository [64x64/16z] {
xT854jn034JHhn7q_KySdApALLfoEF5RXdOFwSzokLmkiGmmJvtXt5ZgNMLWdbN5PtuY7le2oc7fBOQLRddbcoDmAAdpIWYzh21ro2kWt31_hNv9nxoChVHK
rtS-OQNIi5U6hmQOb-I-LzRrH6B7FDZyuonOQhuoZHWcD-lJCb_HUBWB2QnIgvISJfGRMDsFxtH4x3vRK59BWzAWbxTWj_1Pw6XXcWIcRwrkB4aGKKDFQB28
NSz-4jIpFMf6xRpjw9JVVnZGGavBEHozcL-lv8sujwe5qncVr_yeE_za_Ofn0B32S3KZDGd8-L1Z3yvFyt44vHx7FwNkSB0etLT1Tmo2foprI2mUMQ4VutAv
N3u2
}"#;

/// `aws/Compute/Wavelength`
const AWS_AWS_COMPUTE_WAVELENGTH: &str = r#"sprite $Wavelength [64x64/16z] {
xPLNmXiX30P7bVkVE0qHfIs_d9VKpvtTKJUitrB_zL_fAWA8UhuEcm0j-sWYF8Qu0QZ71oh3jImuDg8TqAzSXG8nv0saw5OAo8XNy1zM1B0HtivR0YFU84_I
9W3YEUSTudnhwNW0JkvP_l9ycVPTTkY0owO4Cz09dX3rAIlQregP0Qkx16BfQQFWQQCKXDXcjZmoCq81E7rHkOLV7sx2LzBGYWto4dv_6A8YYAZo4Sa25NyU
FUyi-hwl5nozfDzPAD5slS50dldi6uJilBpN4vvus0uqFmZooB3CFE1RlFEWvk_nt_b41lMh7mAVytUawtVh7zqk8Bf_TkjVNpAitWEmCt8zFs_FRsvZXg2r
R4ZFx_t_eyYs8ZZii1BYddwNAC7Msh0GYgyKj5puUXR72ROSaAKePDARoMJ_Sf3WM347oN9l5xol1m34IW3GjdzLSFAp6ZZeuymXnmT0RcTQT1k_o-W8xAjv
Sx3PyQiz3TrmXVdnxprfaBKOQwOK47FaX94crS6Xl4vUZmYbMp1NOILjUYnmKn4roDVfgdltOQxkVdzxNRtF1FfJ_a_MYQtVKl_rN-y1
}"#;

/// `aws/Compute/all`
const AWS_AWS_COMPUTE_ALL: &str = r#"sprite $AppRunner [64x64/16z] {
xTPL0eCm54JHYUz_nRMXWfD9dNU_i8DR-Pai2ymtUXIzxEXrxe0cdXUyHo_VcvB8ouB784xfysNV7-jD8PSzEHCyUP8zEMGVUEeqpxxVc-ytCOKfpo-dV4_w
_kldx_ll7_CvDldi46NVVQHKt-E2xxZYWHwNF7cIdX_WohNd5o1N0yXrJ_wfVqFVyx-GPHq1
}"#;

/// `aws/Database/AuroraAmazonAuroraInstanceAlternate`
const AWS_AWS_DATABASE_AURORAAMAZONAURORAINSTANCEALTERNATE: &str = r#"sprite $AuroraAmazonAuroraInstanceAlternate [64x64/16z] {
xTO7aWCf38LXFG6I_ljVU0jcSJbs3RCvF2UrgC-TW_rT8c_VZh_edXmbsxrMSfoyumKrsa42ADj923kBxam_vNkclgkoymXY5yD-a8Jrwz1-uWTe_hQRh-e7
lZzxXpZp_k07YPcvh3LJyrctPYPtTys9rPAQl_LPOtQg2XAG1P0s0FgQ62OAy3TUH6EO5Tv99SrSbipv5-wa2edIOBpnZIvoUdF2nkega5bdMAVFdaXhRtqG
0tJlEypED7FwsXUPHRltf0MoHBRc7nVUoTcBEozIYJhPcfpqmZxwFfjEtSy_8L40T1j0_-nDwqichNfzNFD7qceqcUtX9t-OVwUFR8V-Tpv_YxiT99jnj_jw
v-hynHfXWq6tXDAgHsFM5twVlzldyC7egwxHl6dGjxuuVxTlivQ1RTUkV10TdltwBA-4ArzK5Ut5x_a7tY-yoDaRLdZVFcXEVlb4L_vs_WRKzYh0lto7kl1o
_RgBkw_pTWsgT-snrcP_vJz__lxl__V_UyrVX_vRdbyDuYS-gLsWS-wtsJR8Sr_WjoKXTJZtWhZ_Bx9_FoDvNERWE67tyJZKRi-HVpkolqr-0G
}"#;

/// `aws/Database/AuroraAmazonRDSInstance`
const AWS_AWS_DATABASE_AURORAAMAZONRDSINSTANCE: &str = r#"sprite $AuroraAmazonRDSInstance [64x64/16z] {
rPK7hiCu30KVYqJE_G-y9J9-NF2Bjk-aFOiOWQ2IMFyLF3RmkrniCarl1koJ5tt63bb0QE7Ghaq2i1KBNDi4Z9LWIliqVRH_ZCBSjC40Eng9Zmqto9jlWFq9
Fvc4v6avKpb3yZ67o_CtGeev_QF-z0jdlfORka7gBbe1K4eQ_AZxqwT4IybGaMfC9XKcQH9gBAXNVP8Ftr-U7_LGuv97uL8mPRG4h_hWwG-w8I_-NArQjyc_
yPkgeiw-CpNnrwMCbj6V-GRhxUAN5HvqXAl8mVpC3qfI4oTV3OMFDNzlQFlCrokRhM0hAcn5MrSXVTIl_Yx_GT_CTFwK8kpd_WH8QG3CjGwnuOziN8FAOagG
aQuJdcFCccE4w-aJxxC5SrfdiZWpmzpryEVoYp38azqSl8FhwgSLkVpv2fITRp0p_PYmhsRYwWEbjwzey8yxHEg23UYhFueu_5gzJ-eODQux2Nxnmwd3FqfE
7-qpTSCAukgh20r652MbswGE7tgutlY4fn_K6bMP3A3jy5zqpVoDCGku--OckSlS_MLPPhp7FvehDIx-vonVyfpDecUQOizVcBdPYhl-ct_3JzsPMpxJTIQA
BNzDFVntOanWpo_kr9Olo-BDp3yoV_F8Z73x_vr_F1UtV8RfZC_jywyHRYOpZz6mvpTtUiitdtsMX-tFtpnpZ97XPyyX_ypv2Uf7j_gdxvSVykSuT3ptP2NZ
smqcqCDD2ri7cAUjUkIdk3HOT5StmAMZu4k691lDcKgzULIV_kfwLdVN766wO-QVOVhlyYi
}"#;

/// `aws/Database/AuroraAmazonRDSInstanceAlternate`
const AWS_AWS_DATABASE_AURORAAMAZONRDSINSTANCEALTERNATE: &str = r#"sprite $AuroraAmazonRDSInstanceAlternate [64x64/16z] {
xTK5bcCn34JHacHBUllVy7GS-cdsC5M31UU6GRzBpEFJyHTw55kfE6ehjbC7tj0pDf90ttn1QYzsC6ENhnJujMhjno2lnJmNRfitO9qEiOhUknq3x7J6nHjW
t-07UIfysCYXaQTwvdJPU4f8aUct_V9-z8MJQzpG3L9tqGg0qg0XBdj_wIdHqc2g66fC9jMwaKceCQVMFXall2_dbtsiMXxh2596Ix3savT-hjiy7doURwfr
suU-gIhgw9rKugjTtkYtlC7wz-ZB2WywmbMCIRxbWret8WvU3OKFW98tjBtbpKthlxcjna_BCBdCdxAwa7JTF_hj_7xUp7GyfGZxl4-08Ks0FCyXDlmSJMXI
eqYf6J5ymyVzkGKpMYjRFi--28FOyyEASVQv2chtyIlTlQB1D_miukXFHstuSEhYBokdTxoAq2IZA6cuJMhB1wK9q2PRft_QUvWK8O-8DRSnuWUzV_xx__w_
_-_jT3gCh_7puWNvDRxeQu7jSxzTRK3j-yENKq7PdlQ6F5uMDJwVMJpUvcGxgMCyDxNhUSm_7TDlaoy
}"#;

/// `aws/Database/AuroraMariaDBInstance`
const AWS_AWS_DATABASE_AURORAMARIADBINSTANCE: &str = r#"sprite $AuroraMariaDBInstance [64x64/16z] {
xTQ5bUHG34JH4gloJtZFzu1feBLChnckcV2xf3P8hxWuB0LxGSwp4q-Rv7KOW9Aj66Oa1Lj5AiOv6TWY4rzHiryct_4L1Vao9mQ8O2KuvVk24P7p6oTbzsG0
I1QEvJvf95cwCRRTFHi0BnwAtLkm1Swqfjmz3RvvYp307ARg0l480KWWw70wadhptOag5fJLD2IB7kjD5_l96tFpHRdv2YhjQVgLWMIeB2z0KZQlB3Yx6RXx
OJUM5tCcaWOi3xMpJs_gtQCOwqNOTVJ98kKq_Sh4tQlBUWcg73qB-UoJ_e5VRJxYvDjwyr7inTABzhwNPXvzi7Fpx9IDiHmNFsxk_tlroAgAS9rxBnvB4rKy
x9qlDPzhnSC-INQQO4lKaoJjKQzaEqv9aFse3r9npKbzt2SkIOyyEnHxbkJChoh3tIEBvD1xyrp5o0FgsIysJM13loKhhCP-xrTunxKSUQRYcFLu_NUwwSgZ
nkk_UAr67kArsqxxckKlQkPc-WUkVyEr67c6u9ZMUFrNcCfAFPkS-UQr7dbeDFSgPBxyHSqztEt5wVtwURx-GDRdNd3DNtn-EWD6jjqviIKa_O7pNvZWMLYI
fEs3PuU_c-vdW5vnTPmYuuS508dciN8SakYF_KeiibRXYdloRlYD-m8
}"#;

/// `aws/Database/AuroraMariaDBInstanceAlternate`
const AWS_AWS_DATABASE_AURORAMARIADBINSTANCEALTERNATE: &str = r#"sprite $AuroraMariaDBInstanceAlternate [64x64/16z] {
xTO54jj044JH7fw-_uLJAuWej2u8wPj4hyPiuqz9r7uvVMWlJbM6KrAShawyoGwRA4aszefCp2Lrt0EVuyTGsvSQymcP-qBWWvB4I9Sx8eAHu9ArOaj4hWjE
BWks-FLE9Eismx7L95kNmzc-UZtx1i2RX-Bm4QJ0cDQKfwV15syH1gZ3L5qW7Y730y60CH2qX-zELB4WxAOZMFIW8RlPgrVMufFwz46bhVTVcGYcol0296Nn
meQncucd5pOnl9Wjth1UgINrvCkBzlGevl3INMSVhAQUx_zo-_JgCdnGvUnPYCqdO_7oJJzQ3Ww_-DPQV2PRjlllt_VIpBxSV-RYoPJllNzc-wlzwr__-jU_
_lMlry-z77-avolowu9WkcEiaVNHU3qMIp2LrFbHDybq--MyUBtFnUaIvomdjUAUw2yd-6Fw0W
}"#;

/// `aws/Database/AuroraMySQLInstance`
const AWS_AWS_DATABASE_AURORAMYSQLINSTANCE: &str = r#"sprite $AuroraMySQLInstance [64x64/16z] {
rPO7jfim44Mdptlxtt0Y7Nom8VuWfryNrKjHbty5ymNiA3SNAPMTv0VuYrw-GlJJ1Oo4oI9EKcNIfCaoGQPCo98F0F5u_90FIFBn9lwXhoz-02Den7CT-2pY
V5j7oOCu-JDYe-P64TKvImhiKRU_z5iaXwygu_Veejb1e5_ve0h0L7BKr-d9PYNhoa-6CvdIL6NFPva5jLtbmlT6mPWZwaDD4YPvsmyoXg_CdCzk81RkBwGE
Nz0DcSmSpFVFQrzLffz4YQZ8rfGk2jgLFyFfsym4GEQiyeWC_CU-fur0SoGn2htGCQzPO8um1thsJw3bbMRjrQxyQCOf3tdRBvISqGj_ZNNV8gkxAyFM_MW-
KwvcSjiFRW3SWCbjlqWYNMMY5YE3VjSt4eSwcYJk-adQcyoxVia1nP9FfCcE5xdaJqRNHsGroKK_m5SgbtmHYm8vwN8LN_Gd-XsPhFexVyKFEL9BFila6STo
_wF2RH3Po_tVFD9BlcZqxgBya_PNyu7zX_tlvymbdwdoZFLo_tUwwM2RYKi-U0HBlbY2Enswtlvg7WD_U7O-Fw_tdv1zxQiSYOTVbo_W9FnyJSX4ISRA_YVg
8uE-NvweU8ljEm06-_wxuVoIdFib-4o7xEobR_o9UbJlvuyZ-XRvX_a6
}"#;

/// `aws/Database/AuroraMySQLInstanceAlternate`
const AWS_AWS_DATABASE_AURORAMYSQLINSTANCEALTERNATE: &str = r#"sprite $AuroraMySQLInstanceAlternate [64x64/16z] {
xTO7igLG28JX1WxGlVy5Jw2iycMlJfxdNy5m-6uEuc_9uqJ-4fTELeQzr8NMCrvnfjMIOkCkqN6oa6IOIdASBgJ491KkrEBsyBLmePHiUo212wqVz_xCU-Oy
hjnlBhhB0NYn5m3BzRvVisDIxYCYSqOe5U3g3tq1gTrJ3XX0BHWLXru0gJGsqBHjmaK0gJhofULA9IY3Y19Z5dhc-iW7k-XAfCALG4jjMAhjKHpv9ULtwCer
PuFgyO_Tlqa-TymkuNiccOkpN8SUxXYVuZOf6MQ2YcCFZD--5EoaUaPhTbmY6UzundWk09G1IR5ibgIQDRxhdqVYMKQcO-lGHohneb8zx5iqlCmUltyO3Zhm
-6lzxMz_-zlV_lRsCwwV8JKkH15JIN7fuIScanGNEFTzgNETAFdiFaFdOs3FaYTruaNc9pByBNq5
}"#;

/// `aws/Database/AuroraOracleInstance`
const AWS_AWS_DATABASE_AURORAORACLEINSTANCE: &str = r#"sprite $AuroraOracleInstance [64x64/16z] {
rTK7SiCu341HTEBV_y8R3fMn8gLHst-ArHwDSPN_IkORsHKFDYizyUH1yO6NxnG3S5aPjCbc3kZQ762odKEkBIXvKDFluySJdw3782uhTvUlQr9EnTLx0yHJ
Rz0vm1_wmPZ_wMTU8TQUYNVQIAc6CQ-9fktm1Mrtl-2nJD2GoXXDYGHqn_B5AFhcNxlButQSR178z_aLveR-w6Lb80Rsxfqs6zWFyoyV3I0E-lApcr_GrntQ
uHqYRxxzBRbxQURpLxGSFYanM2iVtk2hvzzXH46yV5l1X1tXqWoJfjnJhtwMp5BUlG8zjwo1a6GD1DpzJ2F3P6wuhtsNjUNpk9ZhF1YXSlFx_VFU8hD6PRhj
-nYSArSp-RMFvkYrOV9hdm0LfZ9JMyFuRxr1Xtn84_gtFi2-EFWhdr1oIGVuxpsT9g-iOClFUbIybJM0JP_DnqPkUH6BQbPTeU9xVgKpcMtxS_-y3xbMMvuo
EUTZ-_dl2hUPPsq__uDhOykBnk1LnPF7NyqZt4t_Vy-_tmzkULBbdDNsyp_IJMVcsR3dcskzvSMgppXr__5NYyZCS3ixWtZy_CrW_Eprwo-50l_v-yVldBMb
_BX00PscijBr1CPfgSxuCXD9LjsZczbv8kVRSjvbDkT6o2dD_i6lgMjqZr7fAjVqw-G_t1y
}"#;

/// `aws/Database/AuroraOracleInstanceAlternate`
const AWS_AWS_DATABASE_AURORAORACLEINSTANCEALTERNATE: &str = r#"sprite $AuroraOracleInstanceAlternate [64x64/16z] {
xTS7ZkCm38LXDedyxt_XNSBHr3G5PSlufT6GFv6mq29_IzJywTWt7SbMqZ_rJhQJ7tX13T_814fM4gRiHJ-OsEMhgQsgP3yASnL3NaY2T0pMOPYxt4vzxZD0
-zUNzmA8PttmsHlKIF3dlNtqIQtlLxn275ScNiDoo2Z8dZ6AiekVa7R5DrpR10L3o2nctqbL77wIaxhYr-De612cehrOFVXSyojqeG-z737eZorlb6fY3-O_
V1G5lQBBEvrhNe-VRdRuBcDq_q--_7UkdL-H_NWfkVZ1R7lPUStFhVDtc34XbW_AHbT9HNWtC2cc-z0hNWQT8SihK5rP0T7hFL3dgXUD4TQ5kX_NBaSLqJyU
UjoaL-RVoW_sfp_zwKz_-jDh_nh4Atwqxy1ynIUr2dIV6wkjG-xx0bkJGEfczuBvSIyodiz8FiyysSxyFC_Sr7Rb3-pJKVbhyWi
}"#;

/// `aws/Database/AuroraPIOPSInstance`
const AWS_AWS_DATABASE_AURORAPIOPSINSTANCE: &str = r#"sprite $AuroraPIOPSInstance [64x64/16z] {
xPO7SiKW30M5XCgx_uNpfJ7fyHZI4zQ5kZHN-YcmJC3FPSCarXxfYWNw4zraXku0P0mU2AP9142Zme3KvnjGAW2Z1HmnXjzNV3qc882i-FtzFcz_-zlV_lQB
xMz_--BknXdsJ0L2Ro5dheoCCryHdcKTLhKyu3U_SZinO1xdVY-t_03dIOZXsGOPEYNdli3JBuEEc1C1PD0BVXhZo2e--g_nDxes_-6dD-PFWWhEVMCwV1_z
PsvLR9Rc-VYJX_dhCV-1er_qQ-bRdeRVofUhlW21Z0nVmmD1vRTN_v_5vGWpdpsEK2bX2xSKzV09S5eWHr4O84kFZrBH0CY27e_YSEqJQ3pfKp1Ff3ve6fEs
qZCQJz3eft07
}"#;

/// `aws/Database/AuroraPostgreSQLInstance`
const AWS_AWS_DATABASE_AURORAPOSTGRESQLINSTANCE: &str = r#"sprite $AuroraPostgreSQLInstance [64x64/16z] {
xTO55kD840JHh2o-_uLtMvQUO4pjOVXc237XLq6Tm6kSFIa4E-yFw2aFdM5v602xYvYatIrOP3SnpReTY-x01wfhctp31xnxcnCLTnCy8nuseboUvTRBrZFf
XPqhBjAeG3eZW2MqKm-KG8s0EYMTe96WgC1iJRG05E_tBPfQPAcoGb7g2I-ND7Ta070747gdByj0AioPj_zeW0atI8uxedJjxLwl19AZyjBHcwyzAtDqR347
a7f_-e5SAaYDNdFjfN1JP04ZVj3Vvb-r445KvEZNHU_g1MHe1j1kTkf5CQWJOYuG8q1Da03C2NKJ09OvdktIuoxtUryMCRpiir9mXyYTVjfy_w-dUMHck76-
j-oZK27nTczRLzMh8jxkO_m_LB0GslYYzTsUtQKu4U-kTtllvjqlt-y35r9JVPSJEutkkNx8S3Fpo1wcUgi-2v_g0LhKscQeG2VxXVmFYzb-zrDwmrLCzHt4
aTRq-gymvM2Uq-i_-ogdUeZbtbRe9yjVg0D_m_MlkFAflbrmn9nU_-bA6ROzSQQllggf7lJgNPhCBt-Xsg1Rn_tw-VFrX-vytGkkRElZvGneT-cNUM4XtMrl
dFzS1MkXs_bl6tnNs4FSpm0rh6yL6-sd781uzL4Q3iJhUJ-8MkH-_N4b3-4NzXy
}"#;

/// `aws/Database/AuroraPostgreSQLInstanceAlternate`
const AWS_AWS_DATABASE_AURORAPOSTGRESQLINSTANCEALTERNATE: &str = r#"sprite $AuroraPostgreSQLInstanceAlternate [64x64/16z] {
xTO5ajr0303H5altFt1WbZ_R6uwV6RfAnSEVGjGUfd8QHyyc7RnZirthXxpTDbZZAYmoCyv6CTEmnCRyjEffTkYfb_4TTWt0TpfZxE0p0c3xUttjiyfvHpWL
NRBE1kQdVQkqyKgJ4vw842NQYWUdlH21Pl3z9Dc1PBY8c-5fH02qWkVbFbrQ8sbLyLBAjLN6rKCQ7GSPGFjINrLOmj36ynsbqb0cfMYLtEXLm0C6gJhfCqA0
30QmkZm_tojf8CqAMkZLfx5uKWRfrtewoWblaGwyQ_LOUuX7jA8TsU0TOHzwPJ45NIS5Q441CGLp0SHRGLHLFlGNg6XpJgfOxlLMgLnpsc-slp_wep_wep_w
ep_wpwzVPIRO8MS-jNwVAPRODK3EE2mz4ploufdEJ8TlaDyBuONsR5R2EruRQ-UKwCE4FyPN
}"#;

/// `aws/Database/AuroraSQLServerInstance`
const AWS_AWS_DATABASE_AURORASQLSERVERINSTANCE: &str = r#"sprite $AuroraSQLServerInstance [64x64/16z] {
xTO7hjim48JXsRvp_mi7ea0N2IvqUl8-D7c9tqqLVmgr1NgCYujIS1NyWDtbQIkyRjv0oLOiCfA2eKZ5CYS30vduGBFdrtTy8CZvImmmLm3Y1a0KcpbytCkb
JtOhZCqM-BumXe97_BPd040Nf7FsiW-X0YX4T6pEbuU-jYaTICv-7wAB9Ymg7GpfeXYhxtiZ7KO3ujhxFaIaC0MLIer2kT7bleSKJG_z63gxK-Y0jpUIJJSg
3Zt040PGVUqX36yHZ5vOYImHdFkme27fIOCpt85zg0opqQF7cfcU--mQHTC1gwe4i0-zob06m0j0LAcM7FgBxCGx7lNG-AHVjjv_zUgHLPNXkjvxyLQQgEBj
tcVNVTbIlDqdoGuJ3AA-3MZlzagsuOu4sU_sGUg3uRjzua1wgMU7uigINEg7od3to2Avs7lpNiPI3wXdSrTf0bli1n4H3Elzz5jwnr4kzKp5BUlb_T_ffXkF
Mjx_nQDQwY5UlEYqJ_v_KTleNxZ_xJnSwXc2MrhB-x_2L3RpJ5pfcqUzr4EZULKkw_-_gFd6PgVdv-Vr_GUoNlU28vzzllm1HhQTyyOW97tb_WUnUVyRZ6Id
FwH0SDSR3csOZ4y508dchN9CfzLpFuXvrdp-E9E7y0Vx1W
}"#;

/// `aws/Database/AuroraSQLServerInstanceAlternate`
const AWS_AWS_DATABASE_AURORASQLSERVERINSTANCEALTERNATE: &str = r#"sprite $AuroraSQLServerInstanceAlternate [64x64/16z] {
xTO5qkn640JXR9hkkl-13RC_isG_LYwNuekTOF6pv74WVucN3hQCnqedocTynP6o9GMtN1hdO279s2t9ELn8nKvQdAWrzyTFJbIItHqamDC1om3CsUtntVAf
NvfnGgCnydPXZ_QHzmUl0fZ6PZtusuWRs7vWZpzV-3Pm9Imz-3tIiw_BLQXiMkvgFVSX9Q60ajtxl8rKesLqcxoQBbVPSu-rmc-UUx_7L4-ZX5I8DJtb2bvu
K9aAUXxy7bFkjhsf5wlTxRMlA2LBMKfIbGbx35DvRCye24NuQxzc2gXHGNJtWhinUv8E8DkWfjtQN_Y7rYo-qdiUhpF-Qrt-yfU__EKlV_dBs_zn_upKd6Wq
x9OKfuvVx4nIdE3pvBkwya0rp_upTBm97hEQWxfuaVc1Z9-blm4
}"#;

/// `aws/Database/AuroraTrustedLanguageExtensionsforPostgreSQL`
const AWS_AWS_DATABASE_AURORATRUSTEDLANGUAGEEXTENSIONSFORPOSTGRESQL: &str = r#"sprite $AuroraTrustedLanguageExtensionsforPostgreSQL [64x64/16z] {
xTS5OiD03011dU2a_V-32s4qbkiDmoHca1-Jsd0wp9sn-g0FHiiHRmC1a6tGoq2G2TMM-ou9fIkyT41NU0d0LdXnm5TuCI0MUSxfV5zmpkPxZMDzaJ-diFc1
_gatLx6UtMINxfwwwrJVoJnlPWE019_kAJVR4uCqzPhdTuz5YIIr--e27z04-bBle0rYhV-kuS_zz6E1z_tWXuX1pPz_a0LrUiLCRtNUn-uMPGMRDVvDcyWv
c-lltslIufnFz1gcJGEwY7AkJ_9dapBPUwQSirvK-hRzdEmjOhdVzh-RtxnDAfxvgJtsplHqoVdhkIO_frS
}"#;

/// `aws/Database/Database`
const AWS_AWS_DATABASE_DATABASE: &str = r#"sprite $Database [64x64/16z] {
xTO5SWC128NXXyFzBrpgxSROYSi_6ldMXRXwTKIFwby-N8M9028MzLZbGmcBI6FgWm6Gc4TcLMQu2G7W67a3e56BGW7Om7lhsfe2VjWph7PauCCUo5q-WOdN
NLvd7fBRT0g67X1VQcyzz2dulW0YCxvFFoHdlYgDiOWjgwQ-op1bfevPBHhNAh_iQVpnrw_iyZBpuDYcWp7pz7cr_2kjDSry6T2ncdzashQp-V5tmIBnbUSl
tLIu4pNFUbs_On-xUCmyvNQUDFE0vXQjmCmB0BP_tuSn09dv2W4wOX5L4IQWarXn_5q8VoBnzUSltQrppoUu_iRpm_7po_7pq_7ps_7pu_7pwsl-V_chzmO
}"#;

/// `aws/Database/DatabaseMigrationService`
const AWS_AWS_DATABASE_DATABASEMIGRATIONSERVICE: &str = r#"sprite $DatabaseMigrationService [64x64/16z] {
xPPLOiKm30H7O5kz_u7BAAUmPPg_zv89xHfEVY7__5CptGt0t7EMPa-duLFG0_2Sg-hyGMhDT21Uh2TaU-1Ao9Vwnjh_NDXBVOvzNkmVFh_axzzFybVGYANu
b_q9Cm2pkEoFvdV-bf_xdwdu-6hjSxK_5kEcFoCC9B_IU81bYTzlZRYPVsAiFvR_sFySYa_q_4FEF-_oJpr_6dh-z-UFlvJ_szUVXD7NZm7vSUklllxh-u-t
yydvtmZmxVrEznko_plvvvyp
}"#;

/// `aws/Database/DatabaseMigrationServiceDatabasemigrationworkflowjob`
const AWS_AWS_DATABASE_DATABASEMIGRATIONSERVICEDATABASEMIGRATIONWORKFLOWJOB: &str = r#"sprite $DatabaseMigrationServiceDatabasemigrationworkflowjob [64x64/16z] {
pPPdOiKW38DZo_F-1-wWz8Ktm7HN_pA-n456pl5_HSmCC3Dzb6Gnpokv2U-n99uJkTILwEM5Ae8c4RsKeqNXaIyoeKTrMRueU5Nx2-wONuTd1YsBr_BpP7xL
Cez2cRRWo_TRnMS8pQsDBFWcLz0T2sqULZoYEnrkgl8YLVFBIQduus2DVAWmhjUl5ojgCP1kqZ-fvaVHgmh_ljT_MkDKyRxc_HVwPxL_IrxYp0-0NmI8ck_t
3pmdCcpwnsenoS-Alutlcm0lUIu2ikA3LZX5xP_E_TVySVyv-evLQIlNLb7K-sbi-6zVzX_zVODSyK7_cI5DOFw5_2YcKvTawZ9TqdX-dItN2OLcakAH_tZB
TQpppS_f8UYISu9uaO_pyT4AutAwzb9l_GzNq7v-zrv64zC7yjlMlL3FVwSvJxQpVvLk-CwGPfQyDPUKRtaMRrSgVenl5O68wEa23bLK_h76zFUXz7zerf5L
wN39Zxr0n4mqUbxMZmVF6dYK_c75OyD_vrdciEtbVoZVyjsLcgVtPmppFpFUMQhpawR_ZqgR-KjY_Xtp7qtSc2_a_o_m2rLCBO2F_wfd
}"#;

/// `aws/Database/DocumentDB`
const AWS_AWS_DATABASE_DOCUMENTDB: &str = r#"sprite $DocumentDB [40x40/16z] {
vPC5iePW30FJrEz_uDyDAFDqbCKpM-2pFYmLHX50X1QrPwL2FR8kVeOhD4SjuCl04PCdc_AKoSFMm-Z1Qz4en-oLWXvP5pASq9gzo_7PBuzey0R-qyE3NCK-
5Y9M8eGnUHLOnKxGF38Ztvu0s9HNcFA2Btvuwwq3kg6-zWhTabzxgjscZQ_wnNdnsFYoNnxVdVibClBT3dRF_Q8_xfTwy8KdU233EXhuObq6UoO9UjxNkPj8
z1xbIW4WefvzTCu
}"#;

/// `aws/Database/DocumentDBElasticClusters`
const AWS_AWS_DATABASE_DOCUMENTDBELASTICCLUSTERS: &str = r#"sprite $DocumentDBElasticClusters [64x64/16z] {
nPU7jcCX24Kl76d__y6lOWh6DIDJtbxr1hTTYTVFG_Y4yekHy3sSJ24_Kw7uOaOr4fsbOpBzc536kpuoBkVzrqSVroc8lf-wDltKCJq2gA6NTe_rzBjw-bsz
ZwVUXVrtTM9ymTHePLWyO8FtpOZK1c2nODZuTHto2BhA07-08Ufv8BT-rQNqbwLKiFSfmkXQ88kWrLjzR1hIMcxlXmyghG-F0_y3DnduOeXzoWU-UYpuGVzn
yMnpzeLVbYlxfwm7CGFC6TtuoS7l_zxtSIsCSl3BlLdNpmISOMA8rVMhLBzov0FgTVM10_zhU4D4L4LaWCfyrNdukNNjdz-z_u4-ZuaotzTxyxMVcAbye6OH
X-VNOi6EpYz1r7oUNXNGXHD_vT_w_VjxnI9SgDZY4RRfJyLUvQ-gLx_U5HKV-xoxmcAFifLy-6_-Fy2FMwaNvv_fWPw7lfv-wbs_enD0qp_Kw_dHrBj-qHk-
rlVHhz-4ykZltsVxDrt6gUzJTYOvw7-6Vvvt
}"#;

/// `aws/Database/DynamoDB`
const AWS_AWS_DATABASE_DYNAMODB: &str = r#"sprite $DynamoDB [64x64/16z] {
xTPN0kDA20NH0hg1k_y5Jsp72UBbUFxBbX0A-hlvt_zgXxi1vX4hMxz4EkyyDRKDiDXPtUhAt64G6WgmtNhLkpIp8VKRs7Voi8UuwA7r9dbYbxsrNZchZc1T
zh1QJmfQ1zHb7u0_hKmGEXBJPQ-ASwhctcfe7S4Uz9_aSWEWj70T3Jte--lvTrTbFv_onZNe5wGUZUd6CFza_RpV5VHtzya6Am0ijRuxyQvAWxheOPKE1_za
wuYBNE0Eu6jdINnbqrpGgh1RrVb94DDSP_QTApmUCvpc0bAl6cvvQj3ZsNgoYN4ks0Ruokg-NOCSvm9fEzpuEVnrya7VAUKAiw_z9jxp-VwVmr-G7J7gCur-
cNulI3cpdgrwdVw2R8UUzTYkvyCtI04swmF0OcLLjneWfGWZXlFVpc7QG3xMyD9PyThBxHFvBLSDUr9F7haD-zUNVTNZas3MRsxxVnvQkHnWmNn-xtorfhrA
dSljf7PUPkD-OsT_kbl7lA_z_SVKmWVzWbJg6nrG6l3p_7fH0QH6plKxtszTkGF0MpFVlNwhD7SU_sO0vh5BVrF_-zz7
}"#;

/// `aws/Database/DynamoDBAmazonDynamoDBAccelerator`
const AWS_AWS_DATABASE_DYNAMODBAMAZONDYNAMODBACCELERATOR: &str = r#"sprite $DynamoDBAmazonDynamoDBAccelerator [64x64/16z] {
xPM5jlCi403NO7VU_u5_QSgf9T3qSvaZZSq5rgxiyXUBrgiAgYfxk9vnM_92zNXBrbR3gCpUCxEuqcsfjx7OmwVgrorrdIzUGBNzPI81VA8RK3vvdu1FLsyo
9G2Td3raGKoMA4gM52a7GAxzX6CVN-dEp5y6eB7mgL0vGAEO-rK06RRNK0bGDVExH8rsQU7kPkuUhO-d8NtkYtZdWEuY2t_ijISFv6YefNz3pVnoX8U0cll2
N_B3V2jkb9_swp4TPts7R1iTiBD-Wax6mCgtstk5ahD-1xjT-jf_h1MzZHSVbmk_m-xo1IeBXh-pV8CcP_t6EB8z1a_z5TiU9xZxfjsaXER_CuBnIh-iDxWc
F9w_hnD-KRBlYxsYb-wt1FkFiVmjg-u1sFlnAyWBSNlLv6rVOM2Z_lDC_fA1Zh-dQz_Sz__39hYSHw-7RXzEEj-MzPtkPS5TFBJ0FlABt0uVyanRC8u_ITsI
01E3Kjd7NPVbKCUfqyNE8ETrczEy6eHFbz22YidnVL6tcW0zz61t8tN7H05GsSBDz3_CV7F6P5dWVRzwV8oSDTwo68ocR-FYJTGyBhZf605DFaQ1bC_nKRSV
4k3o1MWF-Gtv5m
}"#;

/// `aws/Database/DynamoDBAttribute`
const AWS_AWS_DATABASE_DYNAMODBATTRIBUTE: &str = r#"sprite $DynamoDBAttribute [64x64/16z] {
xTK5ZWGn34JH2ZYkkl-15n8DfrloC7rXvBpTOUFXfR8Ii3xzgsWv0befNSbRcUR3j_cqxNp1j3QyOLgT-69cDP_hJIEMStocjhMup_qV9o2R90zxxYucbope
xU0U-opY9M9Nbbh4dtm8N0nvwuTT_HpmJJeUvk3_Fna6VFz_HuyV4P-Fhb4Uyk0uxMOrvela6FK34VDe4lDseXBrS4bkfHebLOGzJ9jOS8P7QjHVNe6u7wMS
qGlxNN7lypWy5wPNsX-Popd0bHAkKLffVVei-zTj6hjVclfrzs_SV_tNV_tNVtshqpYypwUj-yVkdolipmVt2m
}"#;

/// `aws/Database/DynamoDBAttributes`
const AWS_AWS_DATABASE_DYNAMODBATTRIBUTES: &str = r#"sprite $DynamoDBAttributes [64x64/16z] {
rTPLiWCn34JHbadg_IyuDFJ0DT3X-viwmvQ3jsTv9I2lPBYKkHB0bVA3VCtTVF2UkzN5PuobvbuDOstm3TtoVzzuAuk-aWnUIfAtYxx8GhhaBp2Fp1HVHrV8
9FZ1BSTHxjyDI6H0yCFhQquoQnxEMmMYHt56IpZihTlahTlYtxj-hQm4b9KCTtRd-JzW_kdlN_Z-XEzVN7zlMF_2_17cdp1_vVclxpzlsF_K_LVV_pzW_jt_
_-l-hAu4r9K-PlvXpWoGvnTga55KRy5RJVHEHgqaY-JJiF0jldJ-vV76dOJYGJB3wY04NpWKfdcKWIU87bPgpS1Htqet7xoNRkr3vXzifOrVlxyp
}"#;

/// `aws/Database/DynamoDBGlobalsecondaryindex`
const AWS_AWS_DATABASE_DYNAMODBGLOBALSECONDARYINDEX: &str = r#"sprite $DynamoDBGlobalsecondaryindex [64x64/16z] {
xTK52iL034NHQ6N_2_wI1aQekntq0W_hXCBXa0DA2aEriptNWMObTQ-rzigcsNk9xuKxb67FO7nF5kXxX0vwblsxhng-ZqgyMgtq_gWC92a6Bdl1AYZ0h91w
lVTxXlsxBnk-ZqgysgnslrMWm559g7Qm2WnQgizPxhSGutkrKDjxjNdlzxtVez_lmSvolwN-5RulGebNc-hQx_VTB_t-nz_lul_dyRu
}"#;

/// `aws/Database/DynamoDBItem`
const AWS_AWS_DATABASE_DYNAMODBITEM: &str = r#"sprite $DynamoDBItem [64x64/16z] {
xTK7ia8n28NXGq82P_yBVaNivcO6U_kdUibdL_3mf4u2vjE_AfTpW9FaIhxLOHQ-ZwTzvok6zV0DmpJjYxQczLolpaZFyMBSvfBtJheA8EgaIzORQRlBjAIl
GNPte3blwxFhd9xoGdRirSYIyUNmIDnE-4eMx2MatjCNKh4hICB3QOS38kNxmPb2MiuBzzu1SRBaF7JtkwiUbr8UZQGtrUOaEz8UocsAf8-QyozlWhoFf9J0
QRzhv6sSX_U5wPNsH_7b3F19Wciaa-RJPzc_rePv-8MfNNV_vltNV_tNV_tNTntcuKs7zV0FtJzNs9yFxXS
}"#;

/// `aws/Database/DynamoDBItems`
const AWS_AWS_DATABASE_DYNAMODBITEMS: &str = r#"sprite $DynamoDBItems [64x64/16z] {
rPO5TWGn34FbW6tT_y2boHRp1bH8zpzO_E78W-LOtG6eEnW0LEu0S8VwGtwlKsBpeqxfRtx55Dzyny1wX1SCwbL_JDH5lv1aakoQ7sG3IfAc-3RULB8fVXcU
aLtnRruH-qUI1R2GvZ-sBvpRPy57umi1qKVnP3HCV9joystAy9VVNsix06q7mtVvVl_hyq_SV_tyBJZ_mlqJxh-GFrB-BSXV8V-5-gFNFxt-wlL_VVvz__dh
-rZV0UWxhCs_o-S7C0Fa_49BChliErz8q_n6ilSaG_Dpwpd8Se_vXp7nHbR9Tx8RAab8lYKtckQZvD05VoYzDyDNtzkKM_wrAVvFyW-smy6lzyyZ
}"#;

/// `aws/Database/DynamoDBStandardAccessTableClass`
const AWS_AWS_DATABASE_DYNAMODBSTANDARDACCESSTABLECLASS: &str = r#"sprite $DynamoDBStandardAccessTableClass [64x64/16z] {
pPQ9hkCW34N1DiRd_tzu50OrpIGuNMOxUjhVuTvIOxNyjzGg8YeYzMrJMm-U2M_wuZ7IWcl2zTRMAKVtPgORPiqVTLpIx1ZkLTUgrjdmkWu7Q4c2TY1abJvF
pv0EB0xeOEMM1hrSKS7B2pZKHNrznTV5_-ch-Kl_nS-V-BSDf9Fwq6h-_MT-MyzVaJb_RcjVYptFl_xab_dNp2z5F5ZWmqlzMVNq_dipAODxVpBMpzW_jKpk
_OJ_nQ-gzMD_yh4lRSS-y7b6tlORJyJx-RlaiEflbiJF23blGOcGURBVnlVb5dO0luaty4G7p-E3gFa5UnRl5vjIvYM4WKN88dvnUJLcebG5hPB5Tv2J3fxE
mHvlJDf11z_z_DDFugY39pwW6wNKji5-Xn8uzA2l_7DfVRv3yG9UlRpW-wDoFTRp_1t9Ljx0zWb8_0KACjlqye7VOBG7widFvcVIYSKl7V1iVYVWU_ihV_5-
9Z9oAqHT-Ddx7S4oskkYMRu_YiFS7Ii_trz1NBQtgE0QShC_mQ_QAmJZhwG8s6LxPx0VB7AiMCLSYe5TX_XHLu3elNlptdkmKOj3B8RJ3-cS6B6HV3KjzF3O
clU8uSLMO_pwcSTyYZ1BVeZDzf_Yi_sd1CWtVgULhv3o__43
}"#;

/// `aws/Database/DynamoDBStream`
const AWS_AWS_DATABASE_DYNAMODBSTREAM: &str = r#"sprite $DynamoDBStream [64x64/16z] {
xPS5hiD034GNZFS_SBcJP2sQpz0dAFb97iMMrVRjzFbc-jMV-MRcU_rtz__Tl7ZXkIT6qpoW3QZhNEooFzPo0QKZhxYCnNS5TpyKnCuND6dIT_wkfxjl3LZc
FcaNopkXW_IVFmpnl34uVsGAkjQWVStiXyT6zkyPxL39-HtV8mpben1XED-M-D4uFzBlRn26yZis0HRbI-OuVDqv7syGXlDpYQ-SZ-UDC9HVupV6hyFhf5-6
b_DdYS_vMeQNywFiBkFNuHNEhxlB-7Luet5-bE6b_54CpXVKu2lgVbbUYmRYEta_vVcR38IXxdUV5i_JQHXU3fnEnESPwhU8xs7YzlzkNhp-_tqxPm
}"#;

/// `aws/Database/DynamoDBTable`
const AWS_AWS_DATABASE_DYNAMODBTABLE: &str = r#"sprite $DynamoDBTable [64x64/16z] {
xTSr4WKX40JGnjdx7_ZBek5KuVBoZc0Cmp6HuO2WmvzuCWRieUELyaeurkMreD0fB-3ul0I5d2L_yzRp_yVILFf_t6OqcALZg85d6C3fPgbVBPrGlv9Yc_gz
UUE5-ZKKnVLR8p_MBFLhb6DTtYWAiy7_lVdYznzVlom7C3gKAjqVQjLlMV-zzRDallX-6b-_fVllU5y
}"#;

/// `aws/Database/ElastiCache`
const AWS_AWS_DATABASE_ELASTICACHE: &str = r#"sprite $ElastiCache [64x64/16z] {
xT45pkGm38RXpn33_G_yWzLkP6sZQPQXRncUWIG_UdfwOjoEhpmsklPviv_b9OvHuEFOJuvZ2B0OFpf78HEBaqnQZ5y6mHB8S-idvsQWoEiKuxKVq0NV-lxr
_v_RKVFh_mz8DDkQJqBMpviDjw5ZcCVodRLtmOmqDdrf5b4L8G3GBI-0-dna0kYE1pnR3cpu02nR0uYR7kN1mx_CWnBAuxNdqrV4HAYJm_ChNmMGGsXEdsNv
wgQlexiVx9PdV7RuNdbynckFEnuhFny443bhTsvwifo5uewlvlonZi_qc8XehtmTDpyOc8arpxbipb_OK17LOTxchxNshL_lBLkswo4-BmTZqrV48iA4QjE7
4d0cuvl6p-qZZ_ZbuvzNFZSYQBiE0Urugq6hlnvkoW0yTt96Io8tYr5J3o9MY_mNUtfw1m
}"#;

/// `aws/Database/ElastiCacheCacheNode`
const AWS_AWS_DATABASE_ELASTICACHECACHENODE: &str = r#"sprite $ElastiCacheCacheNode [64x64/16z] {
xPP7UaCn28I5g7F_2-V3pyXA3wiqpGgLNpGNnayHJVwovYjQ72tPt6boj6KR7a0eVrrY07pndg_rD0FGohsBLynKxcFqnHK0fR4qh-PCsfQK2qfZpbad8-uL
c01lQSLjTOL4kPJXcPx6BTA8kcn01RBuSBhs77CjDpwDahxckKfYwfAHpT7YDIgEAPwD6JtUxlp4cA3rd8Uxnpr_-mAVpi4FtfdPuZBc-_mpXx2Alrq_WgI_
RlsgOSHvaF2rRF0p08Xvsg63SgaRZpn-bq_bvvDk5jNoiQRKE_uR-gtyuGz_-CCV_l27h_bp0jA6zx6xFRuNvYkGsSTbXJs1e2wUq5Q9uBPty5vAXEcNLNFS
rikUNWLCwXtOPVmHFG4
}"#;

/// `aws/Database/ElastiCacheElastiCacheforRedis`
const AWS_AWS_DATABASE_ELASTICACHEELASTICACHEFORREDIS: &str = r#"sprite $ElastiCacheElastiCacheforRedis [64x64/16z] {
xTK7ZkCm38LXLf7l_XTUmH9NiE74PGlQ_DCd-bmPcFwqdynVPXiNl4weftYTJg_q8ltRdct4H3OJ8h7PRi7UmwcN60KvAcPKJWsZf9vZUaQl4H5cIGsL2Af6
WHdHMCBpk1EGxj6OEORddWrlt1i-cBcXUm7Ic6QodS4uZUcffyChzQOV1qvaWD3giNU0LWRO4rzYv85kddbA87pDlvvvOXvVHr-gvhynxVn0kvXfvudRVlxA
ysPPp2CoWSneRaBrKddISzzZ3-1OrEs-ybmu8wBrhyIiyGsl6stAH2knbkZffHS0JUWa3G1ovUkg7YhHhdmWwME9kFO8ETSIkF2P6dMl3h6DLqZpimUw7sLp
KzbIyqW0q-EJjyZiFhZXDET0znbsuaSrutAOm6waNknT-P6egfbgZT8tRxubjUJOyk9RjlECaLvxnOZ_ihTQKh-nRidY1aJzziKZRVTSzsaIG9xwva2p7cyU
qE6p1o3MvfBOUWtmjlneddywC85HB-Owm1DvMDxZWuzEoy-VRVj950VFj2M_MoEoF4zFm-zM7plwzN1SVdNBSm3oqe_B3wCd7bn-b48pnYqFv-bTn06Usd7J
0xhEFz7gxNhFPmAylAunoRnt_TJtypUGQ_rQ_O_xttE0NtWs4zggNsxwLozM3J3wa07joZE0q2_FxwiaqGjN9gfOFMgKhtDKcHcPg9nkfYrnA9lIarZDMqHc
H7DJfjD--WK
}"#;

/// `aws/Database/ElastiCacheElastiCacheforValkey`
const AWS_AWS_DATABASE_ELASTICACHEELASTICACHEFORVALKEY: &str = r#"sprite $ElastiCacheElastiCacheforValkey [64x64/16z] {
xTNNhiCW44NHLxPIplz_xQreH8uO3znUibzi0olLHU_iteatTv8atj57URsX3_VGgtVTSvA8XEV3qib-lrqFzhctVF3NOaZIkDiRLjZYd-1Fy0Rlvx_slEpy
rur_YIX6R1qavp7JxBnDFMahdiCg7rmMlIMlQQk-AQcjUPJ4eeT4bnuHDKMDI1G-gVQdECT9YicJt4Snw0sJKk6JSd-YLtjwfz6pxqazAJ_qyxDaVcPL7lcG
gBmjkrriB_nvFbGRvzl1xo_t-VdbNl9gNpJzs_Ib7kOU5ht_i3Rnqxq42aFix_x-uQDTh-3xFlfo3zLKNltyh-8mrPMyhiPrZ3NDk_S8
}"#;

/// `aws/Database/Keyspaces`
const AWS_AWS_DATABASE_KEYSPACES: &str = r#"sprite $Keyspaces [64x64/16z] {
xTK5akn8303H54ltl_0EWxSz8t-cR0XyfbAKvzVgN__gT5CH457px4kuN3WaNckj0EPPtJFTbMu0MYiUmFzFLWx4WYTuputQ8J_tUdwMG3_tq7DImyQVdt_d
p-z_vq-V_zANtLv_gvq_dx-TVvb_aGSfn_blym_zPnNo5TwhiOdwGj-WY61VUlum0VMyFByL4TapBU1p5cYV3Ez3ra8gVkwHc6Eje1uHfeb-j0AWW1rsF4Fg
vJvg9llSJmhtgVbTPZbJo0lOp5-7yfBF635N_7rT6U4UMJFGUx_TUFQ-a9_ikU3prltttXDpJAWB7ed3wGpR-nH0BB9xPlh-l-3A_AN1k_aRUtailNl-5QR2
VNBlHq7CtPI5lrK83zdQ7-lChCtzVrL2VvMtqwsYLhxbzWiyjpFO1cZKl5PfWDJiAkC-KRjBdySWPb-d2Q-9Ui_bkYeZiwhd5-vV__eF
}"#;

/// `aws/Database/MemoryDB`
const AWS_AWS_DATABASE_MEMORYDB: &str = r#"sprite $MemoryDB [64x64/16z] {
xPK5ijqm301NO4kw_uLR9_DSIFpZCkvZM0jdKhyJ__bFD5CL1gBUuelahdoEU6xR0eYr7fbLcT7T07nFDv1s2HO6kgCtw7L7o5O6ClHR0jdmOTNhX2rVubxN
JHyihdQ3JT_FyKLEjxi2k_4hNJYG0IVQQzD_hIylHBrdrRO_oOp8MSQsV-7VykCRVOblyy5YvU_lRu_wX8mE-KNxAs9rq4tWnBRtToWo0Zi3qVS8LWtBlVbb
HAKDrsSnMGROxlpxJJUxefk-WHvsDZUprtKaOizNQ4DsOMAdcttfvqTw3FlGlFNc9k27BlILRpDvoKi7mdCyW-RAHuwZ7V9urakrGr_NBo2j0lhHm5OtEhds
gmcj13-YNrlTuStu8qAcoF7FVgTR2lOqlwoyIxw1rr7Tu_mTNUZZLPxyl2dUPqf2V_0TM-lLqBL_ZP0zikoZgdCnGwLTB_UX03hqYSrKBWtAyurDVSZAf_re
Yn4wu35UVLQ2rMLciMZ0cMmR-UL9XnmucZa9f1Q4IvRoIBs5xVc_dl_yvnq
}"#;

/// `aws/Database/Neptune`
const AWS_AWS_DATABASE_NEPTUNE: &str = r#"sprite $Neptune [64x64/16z] {
xPPNeY8n30HBqVBz3xpHPCnuPlC--XVAefLG_vlqrbkLRYe28cgUjGjkBjn9l2tJ2c2UhQhtgfPk0BhsGW2FmPe3iO0d-5EtvP3RlCwZ1BhDGqrh0YlyFFuQ
F_U_na_plyYN0UXr_HN05ltF-s-L7_ql0Y0o-d-Pd-XdyYb87kSJWDpBvuaGeWUorB-pBqsg2VOKAKNFBmjUZZxddsT6OG1R_kNfjeA4FkI0JlCV0AXvv1T5
k68tyGloPOtole6rbtAc3DkiSRLmCrKpZz6-3ZdSrw7-Sp1113_OuyPtrK7UaTQR40Tv8R_vbrVyVUdFurO9jRM7GTe33e00A_71RbyOumQgI5Pk-7T0FElP
k1a0-dh_b060g7sHIkbbt48WjlTNSnKOQZVZ1hMs_wgrpCXilQUYEJuJwhmEZkm_ePtNmNw-6MZrX8JzV0600g3xxmT3gnI8cVy4dqxlzmOkg8FtatpdPVD-
4ulB_tUOZFjj_7uxUZ_MvluOQhVtwzbFcOXL_vlqrbkV0G
}"#;

/// `aws/Database/OracleDatabaseatAWS`
const AWS_AWS_DATABASE_ORACLEDATABASEATAWS: &str = r#"sprite $OracleDatabaseatAWS [64x64/16z] {
xPK5jWD12C020xly-n_urvrfgMlgaZNotloz5Zz-Z6vDX3cYQiDOTsDe9AZNT0UGvZv6p3SNus0gW1Is40hgaVpW0XgdV4Ewf-g2dV8X-Zy6tDyNxmvLAZv8
VlqKAhuB86h3PdxkJG2yua-livBGfdEf-FENMJ8gp15fvlEXblq-p_65FUGydmFEzQjd2N6bBvxxp4dyhD-2N-ytF_vwlxbVxtVnRRQz-lokzpTyjXyTNz7P
sqQxhD-0OTe64fVrErT3y8lxLQmXb_SR075XlpCCxEBvNUuNZh_a2ytGab-UNxs_XJf07FEjszyyt7d6n_pGlB_wAfVtL_VJ_Wxesqb_tVNp-VpuCG4
}"#;

/// `aws/Database/RDS`
const AWS_AWS_DATABASE_RDS: &str = r#"sprite $RDS [64x64/16z] {
xTPdhhGm34RXOoTs_Ev_mRT634X4C5ylz_bBpXGWHN-4Br-SIKl69zS9xlSiUYjTAblsXjMDt5Pz6LRNUQtwpwkNkLRzl7wLh_ivuY9Vz6PpJ6WXtlFPBrLe
AKhtZUZkPkuzXfur6WTjw1a9M8lCCJAZ6P2wQKWoUccd6YRbk54xAO6dTj91CYmlQmY9TzPwZ-Yz6U-a1jZGMW1HaY2S7G-GL64GMkdGwl4kZkWVOZpUcMhG
rpSzTk_o4Un7dJAw9YWT56WAM7ozwT7JihOuExl8I9q8shu7pBqrTmFsVIUk--hEnlhOzUsyBwZZtQgH6JdgU4SBXiuqJDVtNuxhL1bUSpIfWsJsvL0wDxQt
2x2UjTr-O_O13AsKWqL9O1mOpDVNIrfBWow1HdURhVSKg00BNKWVkxjLjRj_xgbxOEXWW9xGmMDiSJXqFQFppipT35ZbQnNDcAn5FH5gK-DDtTm_uEx-PVPT
--Ttxz_VUtxutlFBzvwVVjBvRJ8jN1mn_Nxl-YDy-V8A
}"#;

/// `aws/Database/RDSBlueGreenDeployments`
const AWS_AWS_DATABASE_RDSBLUEGREENDEPLOYMENTS: &str = r#"sprite $RDSBlueGreenDeployments [64x64/16z] {
xPQ7cgqW38HF6i7tV-4hd-1GedTxdssilvCJQdXy5VtfJyGipFGIp9horfIJp5nwxCOZrGQXHC2K5fmLc7EDgbBKCj76fIPb1xQ3Psxsyi-KaVZ-K61tfR6t
6NnlMWzFLpwvhbmRPzZd5_Y9N2-uF8s7Bt3kE8Vz0w_Z0t7Q1fvuSdl73xcfS97K2GAlqrUnMZ7tN6rRv0PkQUMcuFjViLn1wmxtdDGxd4qU8w_xGqH5wWuP
ER0K773ert6BEAaI_jxvem2nA6den_w_jsEh3tuiOA5xEtBj_OBpJoUFPUhjZWnwFyvVNKyXWZsX2z7yq7gwfn80Ten1v4VIq5O37FOQGE_zytOjd8jTDgSz
W3_pyu_-xlOZ6RfVVt7_KKTedN-ziijaLwyFvlMd50VGtbuCPm3upmDIXUCmi2D0vzTm_K7L9hKLpH-Rmn_AvMnfF7_4BFII_I_VdEiBwplZVX7MxqVbL_KR
BPV9dJ2NUbM_TTWNdexwVLwm39cswzbGlmLJ7TLlxhBnbHjTt2-ZwyXSlvF4zrifOdZlyElrfp_z0m
}"#;

/// `aws/Database/RDSOptimizedWrites`
const AWS_AWS_DATABASE_RDSOPTIMIZEDWRITES: &str = r#"sprite $RDSOptimizedWrites [64x64/16z] {
pPU7ZkqW34IlAvx__-6dB2FniYHSKDrHSL1o53EOzlbh4bKrLPLZKZrROQXQkcx3KNXK-WPj3VnbkDibzyY6hf0lUFwmeKzTmYLVuWr0oZAtNE01j6M6La2y
lmRgivOKyEuykxUI0_gQtGOVW2ruo1fNVE5Xwxz_vL7-MfZ0DpvngRd9L3xLIslDcmJ6f8bBOogXsSV8IzLKp7pdYISrb_5DTwBPp3FTo2g2LHcklxxXIuJE
_938Knzx2ALP_ivJCv-zKhpXKkeXBq3U1bLtUMrDVsP1t2Rb5a_HlJ5-XVeSyJhgeMD-nYUWpdeKUdd20-tAWkD7B_Tv1x-JdWHiy8mtvUHU3IiIR6tp2iH3
Q_3QxlBPlLjBRlocUvImjSzx7sv32htynLkyYdiKWz3BQMwzglkLFU-aUsTy0Cemt3leFnC6QdXvm7EoE0E9YHVjuZCVwTuKPfuUq0_sVRADuO0VjWqJdtYr
BXdFT_VQ21E_uvwnTS1FHKVt3lY-uz0sUdd6KqBt3lb2ZHs7lJZZ6tSSLDAzC__Q_uTSEUUPiYJetZdF9pGvvfEy54llcCU4dvnV0tZ3ntnOyntFnVbzpSiR
JbSHijuvYI_kByrrUY4MdSS2vs7cKijmCrCpyuXM09HyV9RInBEyxyonTt-yeTN2X0lcoVtLxDS5rkppL_KF
}"#;

/// `aws/Database/RDSProxyInstance`
const AWS_AWS_DATABASE_RDSPROXYINSTANCE: &str = r#"sprite $RDSProxyInstance [64x64/16z] {
xT87TY9138JXKbRT_y9h3PwtRTAZlJbyv8P_ai1lGjGsw5LjnKqfIvxy15_pCkmGQ_9CZ0niArBETmL0L07eyGmnnRdsI3CmX5G0nQQYsJGOkmr07fyU23Q6
afW-KOLEPC_za3Lh2W5CWRaVJm80HcBOsIEFtgX1E_jii2ZL3D2ZG9VBjthLTzAdLveKy-Wx9firTSAx5PbAkTjNutt_W3CklG4C6CKeO07MNhZNfrAdZ-DZ
v_G57BrIZlrJ0PPVzUlrww8ZisXhZwvEj5K1IRtNgq8CPgQ0UEZx6W2JMN68pHtUkEatMEKtzPcosr_xD_l_lPZ7S9FztfAB3eNgwxtr9QlH72LukNUIvOAJ
Uf7KLtivzxRGPh_GF_w9al9YxwJZMfBwUiyoBCIRM_seT1kH4s_rhScFGhTw01fPJR8hNG7Pww_yOJqLrsohPmXM4jlpBpUHITMYTkSVpOywjdfWTdggq4zT
Vz4XV-3y1TTqgs-xNjkTVuS9BiHoT_x9Gx_XHMxr42z-bIwVkFwYvcwcibv_9ULRvWVIy9mzw0KZsNXEo9JhjJep8GlF1SaqZ6M0jlmzmlGX0Rn9iYlUP5q-
A4x6fm80lBWgmqAYd_S7zSYgod35BNa8lx4l
}"#;

/// `aws/Database/RDSProxyInstanceAlternate`
const AWS_AWS_DATABASE_RDSPROXYINSTANCEALTERNATE: &str = r#"sprite $RDSProxyInstanceAlternate [64x64/16z] {
xPG5bjmm40L_axhh_XVESxnU9aqOQi1OOdtz9fZ71lxCZcQJPNfWSGFnM6_N3fRpK410QPjchZDQ-nZaTUAwWQ9ruA3NCRr7C5VtxugXKogO3aanqE_wFFMd
Xb3HQqWPLAqTVobOAbBEA2b9UxxJNtqD5enj-bRKwJUkXjpoWR7B3wcWT_m-rAlzBic7sEh_pEaRw62HM_xX5Yk7agV7u5k-CIg09GL0wTE-xFZPyRChaKmR
_WP_e-_TlVmRVAPV9j7s_hrInrtdgLs7J-xVHH4sC7OkfqXAn-zZFoajkdoHTtjvMaQhy0zzXZ4jN1eaHWQurAZvJFwbwVH1WYkZ4sTzfl_IOt-a0PECHMxv
JGHBITAwYjdm7S2Clf9iz95luJeuTwrbcYpilEtKDwt_mh_1DsXzWw_8lpC__ll__V--W-a6yWf5IDr0iywJnTpM_7Z8Dxj1xqVdLEoG3MEwY66JvqtE7ZQO
5Vex-08
}"#;

/// `aws/Database/RDSTrustedLanguageExtensionsforPostgreSQL`
const AWS_AWS_DATABASE_RDSTRUSTEDLANGUAGEEXTENSIONSFORPOSTGRESQL: &str = r#"sprite $RDSTrustedLanguageExtensionsforPostgreSQL [64x64/16z] {
xTS5OiD03011dU2a_V-32s4qbkiDmoHca1-Jsd0wp9sn-g0FHiiHRmC1a6tGoq2G2TMM-ou9fIkyT41NU0d0LdXnm5TuCI0MUSxfV5zmpkPxZMDzaJ-diFc1
_gatLx6UtMINxfwwwrJVoJnlPWE019_kAJVR4uCqzPhdTuz5YIIr--e27z04-bBle0rYhV-kuS_zz6E1z_tWXuX1pPz_a0LrUiLCRtNUn-uMPGMRDVvDcyWv
c-lltslIufnFz1gcJGEwY7AkJ_9dapBPUwQSirvK-hRzdEmjOhdVzh-RtxnDAfxvgJtsplHqoVdhkIO_frS
}"#;

/// `aws/Database/Timestream`
const AWS_AWS_DATABASE_TIMESTREAM: &str = r#"sprite $Timestream [64x64/16z] {
xPLNiYmn2CL4DEn_mJUdoVZbCFhjactW5Dt_67Vkh30LdX6rM7jo6aU8puzO0eX5LdLNhGm3i9hf0U9rVAO1Qw8dU9-X56hWAt7nYm_yoxSaDl8ltwyJ_tBz
G_zI_wV-qlFppw5_ANzp_pt_0eYevtDovluP_Y0_0Mp7NnyNZ_nrwhyI0P9TSjqdplhlb7B3HwBEl3yWEzAGkEgd0EgHg-gW_eZkTbXR-PE3_WNOP7uPfdBe
Vnpgc_DVoElDptY2x_fejv9dDkJGHwlpIu9gpnT1FfIAN5lxSw7rUa3wpJqCiaxzVZaWmM_N3xPE_2xb2tx5Tm3pN5K7-LMm7FXTBcVpPpQUNwMRoA7_Zkxd
vmJRyKl6_lVlJxcOpS06lg7d3rYG0xy4oREtOpsX5FJeX0e1xo41e9xhzVyRBe2idXFA8PQzHwMh08YOh_vZkNFd4G
}"#;

/// `aws/NetworkingContentDelivery/APIGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APIGATEWAY: &str = r#"sprite $APIGateway [64x64/16z] {
xTO5ieOm303XJMd9tl_2pvN9xowBZF7XvhNrxDaSuQA8uPPb0rm4JEasTIXzG7TZ0vYiV4wmwI3c1cssz9e1aMlu10pLskk8DQUX0rtNUUqGtQg0bcxovWHX
4SowsUk0shkTTniF-dTsRk-YX_Lvc9zfU_higXEJFVVm6NGLiXEviyyWKWTX1f6x-azkOsW6iPzFoFry1f4EsYVF_SvVCBGJkT_rIr3tktxV5_-p_NnZFiYR
3_JyFz-_uw3lNq2CpNu4X5NZTmsLNoopPWsrBtMzXjBNkbv3vMjThw7mjQuRzV_hjVNisIi
}"#;

/// `aws/NetworkingContentDelivery/APIGatewayEndpoint`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APIGATEWAYENDPOINT: &str = r#"sprite $APIGatewayEndpoint [64x64/16z] {
xTS5iiDW20HWVeZi_I_yfBQJOMcaBenKlwqcW3rVjIYgBUCZYXhR1Lm6Q3w4h444I6v5oG3DvmFc-No0vhC1Ctm-O9wxswJSHO3WlDwRiNeY1fJSWUCD6i15
WE81uLM0vXs0FliUG9S2Cky1X6M_Urjz2b3SJNdp593viEFQxmA6743UnjszqlDn1kokE3cJigTYW1-yxpceikSrD3Cw-k3Nepo_u2Yzptj_4V_rxitShRbl
yq0uu05SwX_p_baluh--c_TTwHiOKFfEdJy7Diw-zEnkF7zF-WSRZFHiJ-mVksgY0GdFHyyrmDrJjDznIrnrKE5Lzwm7WEnL-wy7aEpb-A43GljOCO3b-Gs-
PW2Ci4b5DCKldryldvyldzyltnyktryktvyktz_czyTBzzVdgry
}"#;

/// `aws/NetworkingContentDelivery/AppMesh`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESH: &str = r#"sprite $AppMesh [64x64/16z] {
xPG7agKm30LRaj5xzx_mvkYYp9ZPlFJFNtI14lvJUNXG4ktx912-3oXet-G8I63u3XqGG40StcV0OHro1vJt6LyJeVVCEFrNBomRS99mlwIG8p8udnabK55Q
w0MBQdS0vCA7NisBe65n0K5DkSybABWe7QGMsQsBECW1r6a8gYYl_RAAFBtsb0VNlYtPrXng-z8_uajwyMKAEeyvVeq2icshDkuXahuhM130cEBB_AUIG-ay
VkqR7CYt_I1uZT_JAc_qRpf-it-plzc_sT_ht_htd-mhrlrge1T-0kXRONntE00B7vJh-wT15-TF-HmL0ElyvusRjv7s2ba5VPB_0TcMhz7uy3xNlzeRDANF
_a3UPG2b8ZM0uMrKV2RRTr31cEmzu03vDWMaNy5Hmtyk3mzl0G
}"#;

/// `aws/NetworkingContentDelivery/AppMeshMesh`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHMESH: &str = r#"sprite $AppMeshMesh [64x64/16z] {
pPS7ZcOX38CNqhxxNtWh4jgnU1CnDNyl3ZZFCV3ZMyUSumte02YxH8_YNyGTFg7S0h0hwf3hjIx9_wkTmxoR_fh87O5WuUrkV0F_-wRWglxyJJ1IBk3mpB3E
1A206lAzV2iXVzIqhHhKL5ecj4u1qIphD9i7l5nMbTKiQRK3Fa5bOQpmplXZl_aN0uWv0vWD_j9M0Rv_UyGRQ5iNjSazufCT-nTOmyOHlwjiOclTyNs9KMmr
Ljt64_-HHokS_ufX1NR26vnzmBKcMXbolDPgVl3_KVgZJY76mVyDwKE92NRmVoLsngl_3v3K1_o5_ojU_R-Dl_P_nQjHzV7g_rTuOqKrjYM25_yNMIg-vVzR
begNW6XkI-o4RsrBHY8Dsl3ldNI5tQqVG9ebC6xMBuTOAUDY_MpvxxRmBSkM_ujqL9PGBRm1XCeoj_zJViAFN2ZPFqsekRW1sqUThLzNWrPQwG34VDa_irHV
cY3_e0eWvt_xjtWsqSLfBZe9i2yKMVHF0Jl35iekGpT6m1OwyGNxzzlpWwvVzGNd9p7AJpe_waBnlUCN
}"#;

/// `aws/NetworkingContentDelivery/AppMeshVirtualGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALGATEWAY: &str = r#"sprite $AppMeshVirtualGateway [64x64/16z] {
pPS5Wkuw24P3273sl-6d_KPeax4dbz4_xS5ZuzSJiv_GC-5PGR4-fWjWFZu2M7QYlU3CGzYPZhNde_OQbfxDLzuIQhtoyxAHyqQ3DB_o3W3roYj3vBpHokuK
Rq16GRmVduaqYFxRpSO7-1X3zTIuD51JxrC-nerFIfyRFuO5IAyyt_FUUEMswPNgCFa_vV-RSPETpdSn85P1lhTAsedlPG7hFSWyynAlrcv0heJw7A_SIHoM
2dldBJBR_6gw0wx4h1klQFsRl0DLWErvXzZvNt_zMH1RVW9aFEVl0fFQyG5iwsUm5E8ZRmKiYrt_N6rM6_iEr3pqRqBEcR2U-E2rZP_vmTzIhOF3hZN7nvct
eXcn160je6oS-Bz4cP4K4XNrpBVN_2zzhdfzcK-uSZ37T_XwNR--nKUp__tunxVuaTodjFbo-oHlARplL4V-td-tARWsV3x7A34eDo-mpilREF0gnGHllDeV
yhp-AT0R7v3k1UjetxHZTXvgwC-EJoWRLZ2_jdwfdoGvTdn2s8rNUYk5TtweA20Dsh0K-jzHhumv7dW7CW6TZmJLl8gJTkp_K6jv0PHhXBdunFetF6LtvaMM
pq0XdVWk5kZa45SCP_uk2wq_QkfFycRlrZy7r_Y6zuTnxSLMO1hVy8Gzy0PGoFt6U-bp0SJil9g9REEYmVH1BTDxTbMbt_JQQYNLWTWFzT2uIjjFMg5J-9cV
gVSsBHDhpxkhjPzbVUB-HCxp3UpCXttt_ZYLjE_UNzt7hoU_0m
}"#;

/// `aws/NetworkingContentDelivery/AppMeshVirtualNode`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALNODE: &str = r#"sprite $AppMeshVirtualNode [64x64/16z] {
nPS7hjmm34JD_kv_uHGYXhFI5eCfVxQuFXMCEP2FhvVOKDAuC5Ote8obl_YmWUBYzHX8R_0MONDU2m2RyPSnEkCJGWw7cl6GFpS1Ck45_EVMG8Vzny-DJ_a2
wyiyMEgkj_Wcgm1_F0Sfj_mtMh7YKBVuGm9I7wVaeWbnYr-lQOFTpCvRGVaR_h8osFcWLH_uBeRSUGCo2l8DBr3Iz-RA5trK82k_cbU0hZpaES9NVC0fFtO-
-k0bBzMSPLkwyKLr7oUmyWRKTMdZ7SeqJWDMFe1yMpy5UzWun1bazg7-WdM8ZO4vKFAnVcKPed0fxjJ_SblsiTEJ6l2GboNt-Nd-GGppR-NN_9lmJTRclAZU
ug_yyz_f18Xx_9v_0gsynU_Pc93c1RxpNJpsbXV8tb0RxxHoc9yAL08nuxDjahVvT-Wr_a7-jP-K1s2lyyzVy9AFvRpoP_wr2JllTBTzWnyRtq-AEAsozVdH
0bBwfBpDFulU-FNynZgidR-cfzrhOtRMJ_JVn_fJzxCBBrhMbxVet_bDLbpvWG_MJtRbbtt1-d5Pltx1-dcmVX-_FmoaV-tzQQpcXR7aJzzVlrxV0G
}"#;

/// `aws/NetworkingContentDelivery/AppMeshVirtualRouter`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALROUTER: &str = r#"sprite $AppMeshVirtualRouter [64x64/16z] {
pLS7SYGn25no___35wWfYjkU6u-pSLoXRaIGX1u_MRJbRT18Z6IyZiGI96btqOwMod0pCuyijFXzT9hiGR6ynw3LquGrqiIbBnltcrgM111N_04WB-6w1pI1
_7VaWg20aZrWuA190NLk_PbP0EIiYUSndC9blUAud3cJYndRBL8KPq4120MxyMnD02XHKcKOa281Jha-BzPFyrveATOyam9ui4WsNDQ8d0ASrdYgyDa0eo0s
SbGNl4K9OejCbF3BYZcGKnmhqRAshv_ZjH5vs9MLrlbNLi4JNbfHg36wY2SjOGKm_aX6rYJQHXMp_4HjnSOR3gbP3c9GAn1DH_WshecALLsvt1yZ1TJEZ3KD
-X_QNWS-rXW0sHb7I018uHh6n7KRB9aP6F7JsfGTVbgC0cLM64hQddg1dxnCFExZ-Vnvktq1yWA_uyV-mwqMFSLl88oV-BFvqKxz47xd__-trQfVndDi7kgk
NB-xu1clJ-TfhFr3HcpL6k5pId67VmV0NZW_IdIV7tH-5U06uEByMhicwVm4Ol9AU6jj0ND-qldTd06aCZu0e2Q00FYQIA0Im33ClUR08JK8lhy4BKsHyf0w
MYujz5_Fz0NW-rEY_WAr6QAm-8pkpwlxkncQD8v_-Fxc_e4O3keKxX_UsR_Swv_yI3lDlDk_LLtqRpVxHwF-yMx_Ab5retzzV__y_lxz_U-7zxzVtlz-Ul_x
xVtlHtw__cJv3G
}"#;

/// `aws/NetworkingContentDelivery/AppMeshVirtualService`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APPMESHVIRTUALSERVICE: &str = r#"sprite $AppMeshVirtualService [64x64/16z] {
xPK7Zjqm30LNOflxNpZbfH16B6kXz6JdLzCOTT8l_psJP-Bd-nbBNkNFzTrVvjjg_1LYRFpWcVdp_SqSVvV_vdj1pMC_4NRe6vG7SEWd3BLYftvz-1xWPtwe
Vp_kVmAHG4SIy8tV1CAwGP4Rlx7h_XU45zZP-PiQ-u1S-JCgVUaRkaXguJi0DGxx7q0MrC9FdkSV2kZxnfy5p-jVbCBOZUz0sUF-9_GgNVsXiM_EdqDE0yOt
pnY0kJs_XN1TzA1r3dFip__8gvw8wk2ptydUzSD1PtZhT-oQlwODE_6xVfpvNJ_E_AuV9txNZuxCtegDt_KZ5RYlF-B6Nrnt_YtyaH2lhJyZHBLlNJzszQUv
rey2Sjl_lH-QMSAuzMVr_Az-X2XA2Zbk_43q-3hUV5c4d3tbNfhevy_2psc5AFjopxGlIi8DXN1rGtaYt56tMqO1XFOMKIQtbsJ3D1ilOaXK_ciE1nJAVtL-
GcZnNTydJ21Bktg6T_wVORMO_1ll0G
}"#;

/// `aws/NetworkingContentDelivery/ApplicationRecoveryController`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_APPLICATIONRECOVERYCONTROLLER: &str = r#"sprite $ApplicationRecoveryController [64x64/16z] {
xTK7pkGm28RX5mmnzx_mzaMAXJBuxoNFzF8X8sCbNjhjThCXW8xvj3H9zojC0RNVkMC8s7vSZlmmjWi8ilHYUxrBd3WSqJPXnjb0mrJKes4mOiK_CZlT7zLs
gayNfB5ygOgElozOF72WpHzhdkrNu55SqCiyLoTYAf0j2lzbIHF0hWRN8q6AD0UCtk12sVKKwWAEp4O-9tIbU3JoEQAhjImWNkKFfB6v3e0N-O16Nh0mD58x
3xbhd6XrHf9odFCMnjd_pZGAos8j-ryNA7XKmDUpu4M3XaJ9eDYlDJ_0esRGoVl5LQl8cvpcQCvu0AevweDgZlf8wnn5Yv2MEOeTKCpHVZuSr6CVB6y-NJx5
hlNwKRdTRZy1
}"#;

/// `aws/NetworkingContentDelivery/ClientVPN`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLIENTVPN: &str = r#"sprite $ClientVPN [64x64/16z] {
xPG7TWCX34I_6lBzBvmU7dfPjgKdEwuZ_6c3kNsqBbswr5m1iJX5Xz1reWS5BF91Bg17SHa73P2ZeqiEDkNO30oo5FBG7aJ7nz8jmsFdudsX9W3uZjnXSKwO
wPueR74GbNpkExQdd_F6s8uIzhIUvostplteNK2y7coTzs0WKNk3Kai7sXoFcff4PaWjAvCjpjhG-ddnEgHDKd2iP4wtXjQCOudNkZ38OMQpWUP71sQf9h8Z
UjXe-cXUQFluXYp7RoNbeZ41xwQeZJ7pHjltcIDjQSRLfoBpUzyYnzyRME6Rm_oAy3vNwDuVtQFlEp5Nkewy4e6El5W9QUCUCKImMUJRV0f8dkRx9tmdVtB-
NUVugjty_F_pqtJfqgLx
}"#;

/// `aws/NetworkingContentDelivery/CloudFront`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONT: &str = r#"sprite $CloudFront [64x64/16z] {
xPPN6gGm38D_Eowv_uMtP9lu832pBrlrCWKKMuemcNyb_kC_EeSPO3wotsQdeR1uYnqiUFVihd20UAksPJjOpugQinsmUeq-ljrQPcM0zM1Xl49tQRKD6uQL
vnBbptI37tqMjlwghx-mv-hMwkDOWk9xE_weNUXQVw4VF2XGatD-rN-wOGCZz1VP1XmLSHSRK_eYLjJiF9RO0UhIIJMbzWqaBZ-4D71TDtVgzGhoexMnt8MW
OIuipQKoGnk0lcrVF9Ub-rw0n4psN3YXeTn4jsPNQkOqYFBTD_O1c6R-KdHl30HmZn_rFQzt3JRyFkWl7Lh9UERFe__fG9uCw0tVQDd_XNBmrjLrAOLQtgsX
W1A-nboXbWyDI3lhPqJlzb_NrKPq9bNKK_uQhltDunY1tZv_6tyrqebjnuzBldPZr8dx-NFcQwGBdkOVxEe_pB_9AdIdF-1X_f_sJoDTaFVl7-lZiwiZJTu_
EpZKrO38daQ1pmSOf6WK1mT2l-tWB78GxKhVuwfEWoFQxr46dgVpbDLxvo-FwZcddB_UG1WAovyvVxgSFtz3_CT_V0G
}"#;

/// `aws/NetworkingContentDelivery/CloudFrontDownloadDistribution`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTDOWNLOADDISTRIBUTION: &str = r#"sprite $CloudFrontDownloadDistribution [64x64/16z] {
pLTbOgin38jPzx_mu-U1Q-qRLt-sjb5YXdFFq3zu6pKIZOpNNMA9WRIhREzNtNx18mj_uDVPQN8UGIoltQ3rvvYS0VbpSUdBZ_l5NoSIG1o2092KReT2oGK5
b9mN80NKzdfESvRoS_nh1VJQIM-nsFkjgC-78ESY14Ei6jymw7DXVV6xPjwyDfvf061xQF8dmG6_p6y6UPxn9yMMvndV7uLwBhvZa4T1rF--1jaAw9vr4h59
PgrQBE9J_Ozqo2JgCFw2GWxr5put011luSS_lh2QAC9ghRJv6QW3E5_SvHtyrlyp-G8uurFxlTz_F7r4M08jyQjvyxmjyHk0pEhQ_bigj02nv6_1flw6ckPV
362tx6T3VLQ_eXxpAzRwrVLpjiFcryq3nkft_Rj0-_z6KsZ_g1OmcpuOh84Kz8YP76tw8D3-gV_rLz8_UVzE86d_vlD3043G-O7FBxLzwnT5ZJZdFGCOE_V-
-N7U3BqmlxvxVdx__Fx-_U7z-mk_0KNtf_VlR-_V7z-_ltu_VW8
}"#;

/// `aws/NetworkingContentDelivery/CloudFrontEdgeLocation`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTEDGELOCATION: &str = r#"sprite $CloudFrontEdgeLocation [64x64/16z] {
xTTL6eKW441Xfjd_Xc_OtUsSLx_VLY2S7j3Y4eznSR2H5rgGb1x3WlYAthrxzwmTqNIliIC-r8T6XtdtGDRbZM3S0wVFlksJb670erXCw_Hf9jZfWJEiZDhb
1VDpCuQcHyssuD3zAqzHeEh5KWnJxZzAUffQ-2J5wS-Fj3b_49Zt_56hNc7--mCw-Fv1ezJVvFz3qX4zSVnmxzwzy88qzQUl_m8iBfpV5m
}"#;

/// `aws/NetworkingContentDelivery/CloudFrontStreamingDistribution`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDFRONTSTREAMINGDISTRIBUTION: &str = r#"sprite $CloudFrontStreamingDistribution [64x64/16z] {
pPPfmkGW24H54DxzBpnhlhLakksUhVwvF0AbIHZ_i-OFFOQkv5Mvpe9u8igxGmG0LAxmRueh4ohkfzFj_QHT6VdDiWkeC5sn02Yx_N0_NxsKm1gzRFroGm4t
N7OWErpBKrb1TNYgdJMxVLgxJlkkeDfv499Mou_IKvY0tSkRRgwzyzvEv-VirsXvLNwg01Wdl45zVBmVy4AWtlUy8WvnoiUxbGGxvUsTuv3Zb1y5RtV73tWf
O86Ty_FLjAJ60IzLGpx25oLEdl0BBfvuX0ymiVzlyTVQt-Or_-Vz6y-St_Ftv_dxg--FfV2Mrhm_UW1Mr7l-cZ9zVni34ke3RoLx2kJx9G4kVeCRB2b50dpd
5RzCcvAB1FZE2wwV_83O1lZEAwv_7C1s0HnSSFr_3T_TuUHIxfPSMWBTrU6-EmvC0mZURdQeEviYsoQxjW7M6dt_r5pQNdALjNyymzKkF-fVrGFIUjgqmTRm
n5d_hayWhEdT9JkL5q35V3yvem3AEqeZGAq8zuXLqD2DvaAqvZZHZ2mkLSO57yfiJhFnF-ih
}"#;

/// `aws/NetworkingContentDelivery/CloudMap`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAP: &str = r#"sprite $CloudMap [64x64/16z] {
xTC7ZkCm40NHwa3sl_-9TpcqHJWGqWPalSbZbnjcqFzrkzrkpGtCi_HKfSkIGymi04ANb8Cbg1YwLCOFArrWM9T0WcxGv5ZzPhZE-Hm3yolf7HGGWgupEVEZ
Lod91j2Ir8c2hfTU2bgGIJH29mAl7wp-fmHQdC_diFhhxryTx24UVPMCDduqJ2UIq67sy0oDTg6l1qdmM0WZfJmVNyx192QfGHMdumF09f3YkH1o8d19bjfo
yBuMKbeAeERFlf_UjN8VsQ0XcNlVVVS21RMMaP1a3AR1NPDZc_6nVft3c2HLBPjjDFfQX_Qo_p9UkBw0qb06NMMcrVFAllV72IvrM9a4qWBx-GrSwZCBXhZM
E-svy0q2TaLX-g9XS-7dNk_x9FGD63GfeCt46Vobtr_31lHZzxRh5zZ-_FVvxUZNBo-dVCCPIjk-afbl9AUifBrAWpsBhl_ZThlTVW4
}"#;

/// `aws/NetworkingContentDelivery/CloudMapNamespace`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPNAMESPACE: &str = r#"sprite $CloudMapNamespace [64x64/16z] {
pPUBSgum28I51CZxF_4fuWwwup-Uthkz-ICV0Y4Pk_rjgRp7sPtSVIrd7zTN-7gJ4au1HlkmPEsQRR6r8jQojipokIqgtjYyZ_mqrWgH_7soh37N2cRhONxa
5pR7JDzyj4HQvfWMH-wUTXQYYg-Jx8WCcI1JmhMh4el35yzPfJ1uS0_PuXMiUHCLVXhtxjgBxShVLGjHHvvv7Yn3j4UUC-vgtiUZOoXcRE_XCSdpqy__UFtZ
RBJ_xxzCCnkI5zTkghJy5Q-nIYuqlgHxtXR4BOyd7iU9QHQVLvC6qkqa9Hv3cuMhrZxNQm6y9PcNn8_zFHE8hINFl6NDiagM7_LTVAoeVJDc9Y-uY_njd2J5
-KZjdhk_vBQhwue7pNLL55y2FpQjTKisKtbOlpREeqAKcMduYiiqQmkXyLcAiHhKlPLpZm550L2dniQ1-YuhFd6DR8Y7TQ1H8JXZKtATXOdf9fMNcFj8fjCt
-1C-_909drCOdmuCMKqshH_xuV4eZ77maKgUskJmecuxpiBR_H-uEVR76zpyX0Tmue5reBbuwmJmUnxZK_CZHpAAyYNG8FN1VA_qJiIUZ887xeM_EBzmKcSM
n1CEFZnrufEjpyMZ_eGQ1m-1nzquLbFG_zriA17y34Yluwczx7KnBhXPtMB1VPPZhsfgDhuHJnBUD47u0uy-1ByE0MzUIWpp2vVLt047RmiQZ0UgxsqpssWl
rwyxvXUBmU_AXwM8r-28lGZo_FuGLH1BAeMm8Qzu6HRz6dmV8HLnmskoJBF6nQbvH-WmofpLIRestwKzgvjKlFWEJs8VfFZVQLOkxu8xGaJJxzQWuNs_S7Hz
M2BRtzO_
}"#;

/// `aws/NetworkingContentDelivery/CloudMapResource`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPRESOURCE: &str = r#"sprite $CloudMapResource [64x64/16z] {
pPVlakGW2CGF4F7zd_ZArVjTiKOhpgVf_Jic3GWq9d--4wAgyYdPQXk8ye6Dqfd0BoreT5OjPbQyTPIhq7FKMhknExhdbHi3qbgptskHJ1uHqDFgsnmKF2qH
oBleuxacFGNYqQBQsVtcScsDbfB3xcCVMQVME1aGqlGOG6Y_fqUohxtk-8hRkhtBMtkNPD_6-Av7OiUFDtn_RWRfOdihxsfFgza_-ekIbqOjDSUER1UhYrqP
uwbaUX1ITbNzGS3YoK1PGa-rHZxPD7qWkwQSZXLTP0EnrGry_zNO8abjnuxWGlaNgfCteiFDfjHLfHTXccCUJsjdtS5eDgs5TME2TLdIO-xsdozrqDaDTwFN
nDVXK9FSQjelkuKlyALDIAu1-bhySmlpT_vU-HBid_nH9lxXJ-8FRnE5Au5RE_9dwJtLt_YNkU-jsPpscIyngw2aLiP6w1FqrakcIV-gwzYMTDJ5D53O8OOd
wZeOF5jp_6dI_p8inBmrw3HQpXVw9n_9zwfOVU17gm-3tHY92Q_vOh9bd-C7x_Za3trur2Al-TGFmQ7E5QjUgQSHJOikelFXHWEi-ktxA3J3OLvfvZC_-BE0
ogew8LxnAJGdD7o2qrFyz0k32JwZCGBPRfuW60CSb64_Vu6Udp-x-q6Yub565uX7k9oVVq6KKV6QdXlFa66Wp73dZAaNBn7cjGYjorY3VW6BT7vVWq5krECI
_lFwfPJy6i6HynaaqF-7C7ScpbVYBm
}"#;

/// `aws/NetworkingContentDelivery/CloudMapService`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDMAPSERVICE: &str = r#"sprite $CloudMapService [64x64/16z] {
xPPL0kGm3CMCiUv_uOLXI1eletu6-jGmo3-FSKp-U7_aafPlikId-PvJeFA4Rlpa6FlnVhUF-lv__x__t__lgxjUqf7fAt-B5IVywtpZYd-L3wIA1g1jtzpj
d7R8svUURuCJ5I8oRf6YMdvoenwjX8u_W15cbaM8Z9i_Mlw0KZdZ8XAtMGHZxmUKFAAGroU-zm5zgv1UdiZM3_39LUNy9FO-ymurKIZP-WOncy0bepN_7Mmw
8GRKV56yDb_b7G1qwsiKo0G7KjJr7iqJJ_a0vlvv4ZeCVMlPGmOgLQROfHROoXzGBeTu0VhYvva1GxRaIogvKz9WmCeFQU3WCi70fO52zFr-JY6RzJUPKT3i
FvKPqHe_MwSAolQrPvafeDBNH1sFSh8dv03Duf1KsQ1cli1KVXS-0W
}"#;

/// `aws/NetworkingContentDelivery/CloudWAN`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWAN: &str = r#"sprite $CloudWAN [64x64/16z] {
xLPLOWGX37qnmlqlN0bIPzb9VUVzZ41Sws_5YHE50Iw7oHK1FIeTK7S53ch0JRA23n1xhO376n2Vry18Nj8BsHMo2P2N-Wj0lkyts6l_6NRzQ01FDO0fc07R
aox1XZ71GIWx6dWdRzJYBer3Pt3H1nHN2m5Aa4P52b09fdJP-6O3nsi0iCf0wQzBK5m8oJpkMWiWnIkjhHVfjai1QWTGfFj0n7S2ICrxgcQgpOC4s7xTkE05
nAk2jzg50zqJfl80jMuruEt6kbtqi4tpvxk04D1rku2zJs-O1R5i5_38GweJr7yj6yeizmN6ZJHzNl-y__BnM-SFCn50nBoLFnCCSoMy0AdRxXomGLEAr-gb
40BDBtlrAyECeRWiwZVRF_BzAzy_y_qxFp_oyoi_F_FpExy_vFUN_Fxqmd_9_J6xl_vMd3Xn3m
}"#;

/// `aws/NetworkingContentDelivery/CloudWANCoreNetworkEdge`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANCORENETWORKEDGE: &str = r#"sprite $CloudWANCoreNetworkEdge [64x64/16z] {
xLU7biCm22l61Ft_3z-W0_FoCdNxgkuM6Td4ChtyRBJ6i0q0x3JT4V3psLtLJokuPNRWtD81-lwgG6Fvf7vs_RwbVg8J0fFASROw1hWUO_SxB-w1Vg1omR4M
DNn_jXYh7KaUWeCVRtI_1Cl8u0-ZsZvwS9AVxs5xnBjSAly6yUKfZ4cIdlo0fBGbU8reiu--eRugL43dlo-ZZ2-6W4dHjyokwJeUwBE8nTcNEdfNxLxg1lYY
pyiGsF78fAC20oJYvpu_EfLa7Uk71kWjVkOJKygtCayTee4fVQNw5825QVhu2zBNga_6zu6iSzRzzfCZuwjFeo3bB_ocG6Fvf7vs_SZwaTSFUVto-uVVlxn_
yFx5-oVlt_pvmPzV_Fd9dzzy_y3tBtp_nFTlp_x_sV_p_n_JvtykNW4
}"#;

/// `aws/NetworkingContentDelivery/CloudWANSegmentNetwork`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANSEGMENTNETWORK: &str = r#"sprite $CloudWANSegmentNetwork [64x64/16z] {
p985TaCn30G5Xjp_meKDZ5q_1HvsWz_MYEt_AjmaZuzHpt7vaUl_xx-H_Zxz2qiA7BU7aU_HRSfy3lXnEsfln9PfJxU03_Fiyf3FQLVcpH_SykjDLRZL2j4y
BBma3s6dk9vkDMy-2N3Yt1KyHWMULl0eSN3Wczqwlzql3b1ikFU_-zFzC5tbM_ATTM6X-bQ2x_nOSefvR4hgA9PQXqbud5nYYcARt5IsQ99Gn-viSAVSkL-r
OWo8naCGlkXjEQCrWSJPh70feDhIWV6elbVB2TLDSaRW7EA_48qcQ3vIwMHVLz2Xy4HJIj5g2e32PIriy2H-f3HqKggmZWkQDKS1A4o5e5PpZ12vJZpl2dj3
yJ1t-sw_BWe0Yc4WY9xYN_9d4F3ARHGKTsVMsFdf_UdxmVUh_mV_RsX-_4a0-GNvYVcD_Q7zXVsf_OtymFpY_EJyvlpe_EhyxFpk_k3-ulxa_lPNKlp--6n8
_BNTsG4
}"#;

/// `aws/NetworkingContentDelivery/CloudWANTransitGatewayRouteTableAttachment`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_CLOUDWANTRANSITGATEWAYROUTETABLEATTACHMENT: &str = r#"sprite $CloudWANTransitGatewayRouteTableAttachment [64x64/16z] {
xLQ74a8n2FEHuvh__s6tuWORjus-JRf8Sym-6sOAsqdIk-cDQxJ-tXlGkaT0jkqe-fQEnEeAqbI-E7vr_SJygVhf1qZSHPXm12STHvXkqvbYIdAUmHA-umOy
uu_lulgbC5K_YNuJNA6RrVaHVxnFo-EF-6EVAAvVn9_wL2r_4N_gavNyEYB-rAV9aZx2rrM-E7vr_SJyYVhHz2lid-7_7_1z-BzGlmZrqpVK_wF-7_L_-Fn7
Om4
}"#;

/// `aws/NetworkingContentDelivery/DirectConnect`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_DIRECTCONNECT: &str = r#"sprite $DirectConnect [64x64/16z] {
xPK5skmm20072UJ-5twoJYtvNMUbEi07bVxv-EMNPWA8n_DiRiWp8YYNp660FqDl_K8QsDDqGxBVYITMq23x8m7Hvn5QhpYIhg9jBhriX3mZETEyjX4JZMo2
ZdqepOjpqDC84jf8JoXvI-yQDlRbCPOgvEEEGGUS1z-nHJ-0eocaWDoFst_LyOTsAA2xEaZKKKQ3gBFO7sETJsHNJkcYH4ZTgI2Gsde9XnO0KSkCqPMJ808T
BcjPccpRvUUgzU3e_pNVAhG2VTLwC7oL4KP-Go8WuBHMxydSzeLOt7BQuJg4SVrUpZBmSqr-MXltB-jvKFx6y-iDsjXFGG_kdZZsmq3o-BqX_rZwSW8OFCvT
ER7qkI0MVK3-P-ddcJvDzTyTfs3FUe4_Fu3XVNp7pB-5VfXl8Fe8MCxhmXRJ0H9YXUApVg1zXNy1_xtvvPU_
}"#;

/// `aws/NetworkingContentDelivery/DirectConnectGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_DIRECTCONNECTGATEWAY: &str = r#"sprite $DirectConnectGateway [64x64/16z] {
tTO7ReHG38JXl6RF_I-SDgIm-f4CwHdrbJylflhkbwTzIFVKkjwXBJrbNkHJMt9Sq2EVf9GfASuEqSB_AuMFOvtYQvlxsRkPvfd1VSZslWvIzeBTrze_PRXd
RN3vXDllPbe738lieJM43zAUHpiU73oSsdtu8jdpfTs7Rn9xlhNsIHyMUBwru_OtToGFcS8xWJtdx_f-VFTM_82MJYh8mvLcBlHOL-FikPAx5l-AENYiPiwU
wrFIe4WvA5DIxFY5nCP3rp8ttrBAZf6YKwRvkmRhpFFJDyuDs3Fd1koPSmFspBa1U-RSW7sQOuCi-9Rcq23-yFktLJrNJyswRqkt3NXDP3yb1QsBqSbJPNRi
gjNGyzUVDn3sl7tm1iIUjn_UmC13zy86Q5_R_gXG2cXRt83jOQpg1cvV6vpNH2dAsp_TA9BAW-ED2eFNCuKtQprjUV3_cmS
}"#;

/// `aws/NetworkingContentDelivery/ElasticLoadBalancing`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCING: &str = r#"sprite $ElasticLoadBalancing [64x64/16z] {
xPO5beGn28C_gD3xNtZTb_V2odWpBZX9_tKJsDYO3Zv_RTvvGlzjTEWH7NwPWJz7xlZlx24UNm9EO2_o_-V-pMPWtkPd--By1c-myV5Rh-nF65YViM9s0u_r
X0QjMEN7eFRITOTHCG_mBrstM67LhJFyszXyWaN9V629IyeZpweSTSfDBArf5YMWfP4wJKX04Yd9IgymH3xJ8eKW__eLAOLrV6MlwnVvo_wB_idvY_cf_HD7
YDX_iR-IVoRu8_al-2lrH-Y7qZ-XNqf_jNvg_TVwhJ4vZ_D_d_ydk7uJvBzKR6pS0m
}"#;

/// `aws/NetworkingContentDelivery/ElasticLoadBalancingApplicationLoadBalancer`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGAPPLICATIONLOADBALANCER: &str = r#"sprite $ElasticLoadBalancingApplicationLoadBalancer [64x64/16z] {
pPS75gCX28J50hd_YVUFP7pkaC9kwhnc-rGGM_bbrQd_GuUUbdJyMoSDx2PjMRexO3fwEzI7cfVqFArDza9fckkXscmciKPcnrOV3zxJjImzwaUvet4sxIvU
7zXtfmCxp9FoG79kTMjqtKO9gjmkufkPKODRiczjTzCWERyCxLVy8ZjVI3ilqS-6aWnVAlkmeph7bq4Md03aU0Owkaln8AYtFCz8GsSv7XDeMq08yvt4l6n1
j9Av-5k3OZKnbTsWgDYchj6aaCJL3anSeSMQaIAOekgT0y5EIkOdUNP0FBDaN7IqKQGbdcKmf9US_pa21ExFyhm0j_b2-bcU_JVCxB3MJ0L5efSY6z5_yMY2
-jOIeaEEuiTvc_gRHrdb-E7uhMGLhNIb-CN-IVEqVsW1iZpSJmvey98OOWwnQ8BAHkRFf23XsF9uAKXrL2gTNtv-WlUhhi-4EM-phADI-Vp4OOmvEI--mXuT
Cdc3hK3eDaZp86W2qn4YpbSHTLv5wkIzKW7m_Pdt7-vFlhy7UAoLpPgfjYgt-vlV3tsgbT8yTHJ9AkikUZ_G-oKZxAtuVaeezG3Buy-_7vz_lpx_Vdx-_Vxy
_-7v_ylp_wVd_s_F_n_Z__MNzGS
}"#;

/// `aws/NetworkingContentDelivery/ElasticLoadBalancingClassicLoadBalancer`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGCLASSICLOADBALANCER: &str = r#"sprite $ElasticLoadBalancingClassicLoadBalancer [64x64/16z] {
pPS75kGW2423jDp_n0kEBZ4iJjl6UrDY-8WDyFYVfNNv36KvVuhmUqOmsB21hzBq05IOq8HOzEn2hzE2S6q4bDSiDErgaDz0DwpjUUVqujiixYJlxBCFROlZ
0R-KERwD0RNX0Po5ZEK3rwfw5--WDX1w9UuyKn9-Qe26MyQtR1mM3pIlWhiDMrx309n9IMTBoHE6WNQVGvhhlk61mW3VHc1lZZq_fGMmT4zR_ZYdK2B2Mk81
YGcRDyzbf8mSwCsP9nrzAb-c8Lp6ozQ1nAFH8NJduRA9uc_mtFjjS43hViZAzqu3mJlFCLL0pDt0okDqr7NXncD46ZmTDXTuvS5QfYTwh3mO0ilAeuvskcnP
ISCVF1iGRb9xV2JuPGAc8y77zcxkkDleernu7lni3Dxq4UHKQd8E0Nz1Yoh5YSMLP_j4tfadt_vEVbc0p1l20nGmHOGnpI_pggUw0gy1xdMUZTMHRvfzDltF
nchyCt1fv_FyvVKZ-pGr53ZctrY-hr-nVw1RX9akc1npesd_5FjNuqGurZqq9zB-hSvFnDUPZv0DJUUdFByIOsB7bUsJpsyTF_hi3KMHNaj1ZXzr_08rWk5I
URGSlxRnKnnI4T7-1uhuMSTlr9pyK_pUve_6rhagMqFmGRoUlp8VGCwVUpxdpvo_ztpEtxb-gFbKFwJwHRWKurFzakgdhT2p0aotk4A5E-clIydrO_QWdhnK
ltvTFtzVltz_V_Z-_lBz_Udx-zltzyVz_VL_bXy
}"#;

/// `aws/NetworkingContentDelivery/ElasticLoadBalancingGatewayLoadBalancer`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGGATEWAYLOADBALANCER: &str = r#"sprite $ElasticLoadBalancingGatewayLoadBalancer [64x64/16z] {
pPTLmYGn28IRnkh-5rvXMcia-PUrxsaaHOmWo__CMloQr0C7uLypeW4meRDgGv7XfZymZqHXy-fGMIw8nfo5DMjOYUz86SvrFBbD_yi405y-u03YetmTxE-3
WGHIbW6IGBwRdIsFnutbaoEjbZL70lRYN8PxjonCdiyvwvS1s5XMNgozldUv0o0JGFc1aUMN1yLuIPy4TZVdcvVZZPuKRCttum7IhjJt15YYWDtacm4-56ni
IzRBFeNaPO3rWIBgQ2zETFCdHP52-X8LXx7JTtGvKG6yme_P1v_GmsESNA_c7IB7g40k4tec4RHzqilm6JvD5td-OE3TEme_lkqRiND0xdeZdwRxhm5o7F-g
1O0iVSbTYuh_Fm7IFO7x1PG-SH0_zIjEfCOS_lyxzTtrTy-lUNzz_-dxR__zyFjLmFNdtZDzulro_7WVVyGAsRTFyMkeJnH9yOlYfturVdByrZD-8wy08H80
IpX_Q4pbZs7-qjpX-LbWKrUEWlFdE7_p-LF-7jOFVFzKFpJgbtxzrA_V-lLZltxjry_z-htVF_Jxbtx_rE_V-lqZzw__Czy1
}"#;

/// `aws/NetworkingContentDelivery/ElasticLoadBalancingNetworkLoadBalancer`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ELASTICLOADBALANCINGNETWORKLOADBALANCER: &str = r#"sprite $ElasticLoadBalancingNetworkLoadBalancer [64x64/16z] {
p9I7Oa8X34K9cV__muvhmLWUkCVfhfmiyYo_J0MFgMhHCRql25iC6DzgIm0t5JuWQXv0RhUDgII8xRO8rN6CoW0XiDVhoUMHLy6nI2qBzDZQKgzN-bi4y0Yd
SWKwdffa7o9FppsXJquAobhjyn-4HcLZBilUew6bq3XddqsUHX_P_7_rEhalrthoDVtGta6FFbyOECaV1PckPJq5eEIFnZnz8OEVyuyApzAJ1h3ZUTS3tPS8
JWj1KzjmCiyVK5gYxSrhpxiYkZQ_dfFsEq8fT2RPfR1lBYRtK4XG8yDqjBaieDwAXIDVmeh2Nk5XlNtuqhYSXw1CzXD6QUhDL_YqEKz1dyFyqNPhHDP-LFX-
8Fam-lW7dnkE06d_yWNC_RZWe_xjzJyxloVlxzdzUNP_dtz-ddz-r-yVTiFxnxxuEY0000116DQ_jI4cO0ZPuN_P_yJ_hVk1-wNxYVkjVa2_e9_GR-X7z2lw
6Vtk_U3zulta_URzwFswlmC
}"#;

/// `aws/NetworkingContentDelivery/GlobalAccelerator`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_GLOBALACCELERATOR: &str = r#"sprite $GlobalAccelerator [64x64/16z] {
xLG7jkGm21Czzp_nFZ1v_ZMEj-_-Q9e7Zn32U8w_4myUk30185RV9uVW9IJssQGUSOIR403TeDjRnM00jrqemD4D8699eZDWz-ZzItrZNsvwqAhKKD0u5nMo
Mzwzn5Y7PGUK0Ucj1GAmji3GBeYIofcLLdo1bBfafewbh5O515tAHtidi4U6d35WwMKjdoh6U0qsIoul2W2Y_5ObtFFFQM0KxzSTg6s2rPBdH9m7KGANvUlP
DuEy2dSV-N86KiY6i4wdbQlJAEI2JuXJJAPGUUaqTib7yJy7PwfBVdhu77G4QAbltOgtzVjIdt9IVy0_myx-oqVzjyh-EUfqBiu_Kc4rVmmTIpTLKJKV9CrD
O3d_Nc_yvlhaNfkxWEG-Wv0WKGQhzBU-6wlxx_a6LWNtAGZ46BbZop-WOXdg7lKmrC80Myed0Caia11mySSdj8mTAtW96O3a2CnoLK7DGfQmgZ4O0uG4Htdp
umPQmmKdnEVE3jm8R5Dq7tL--mYyns_aXm0owv-1co00u3fBaC_0JHWeOenxmcVW7XGwAaTFxndOrGUobJG3c_wfzJkmqt_AThV_6TY54TYFlnKF7dm3
}"#;

/// `aws/NetworkingContentDelivery/Interconnect`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_INTERCONNECT: &str = r#"sprite $Interconnect [64x64/16z] {
xPQ73jim34JVKQRE___37NAWGc2WsCNelEnGnqtQv--7__YF8m3K-i-ncGW_PbiGwU_Vck1uJjTqs-sn0g4bbeSA3dGkIF03x-lfHfn7HEGTVgEL_tX1lXDz
ho61igVG7VVF2iNryJcJdNWWh4ZQHWgVp8lOydLu8m_spT807PlSBsnBVhrFz4q_D-IrQiWMa6ixlbVGmMiiJKYeGLlFtUWyGPuMwEgVHVDR1xIOlvnsPfsf
Jdmkxd4TSg-DPzZpfmC9wFk75j4TVXVT884xyevV9wy7Q6Hql07X4lyc_rSV1dcUIOmzgdgmrdyjqSXXqCRNzd6VDUGb-sR0Gs6M_YiGoYL5rXNuB9apDXDc
CJ3iXOOxFYqKmn-0_O0U0fYQDBoQknCKnzrBKFg4b-lzK5s-U_gSY4wy_l93-XaCn7bNWLj4McYvZEVK6f5REXVqjjKxm7OzYvO9o8LVfHL6BpCKVf5M-BNx
yqUg711-AwreyxOZnzcb_ukqGgnXNwFdHXJfVd_C5JtVYtHxqpRBDFktIdSAvZKk25kqAhq7bxkMXt13pUVlYl_uZwy
}"#;

/// `aws/NetworkingContentDelivery/NetworkingContentDelivery`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_NETWORKINGCONTENTDELIVERY: &str = r#"sprite $NetworkingContentDelivery [64x64/16z] {
xTO5RkWm48JXVzdtl_33qgfnx9HfHD58NzXGxvxnZ7nT_-FB1KJzQRwKszWJV0AUOrGew6MVOEtucY_mThAwvgtVigDroKDh2y3sVQ2zC0IaThqIZxeITDSB
UVXESzCVls_7Tdo5mV4_eMkV2Y17Ff38XNV0FEhO0_YP3pWPRk4cGCuzn3XF2EJCEpgMCPXv9SSwYayyZ8qayYm_OE9bw_vhwWr_rlqdrDeRDl53aLhnW9hv
8U1rWYiD_6JyACluwVXDapCiLjlhduCf8PENil86CPGiWRpk5V9cI2Z4HLy2TJUa37pRXuA1r8CXwQ22s8Pt0D0-f8F_gPLFY57QVAjdleyXg5lVwvLNydue
bu54htUlV_CH0E_r_lChMYCWMxtt_klkQexrUltznHvCD8csUdlyjEjV7p_j-P_agqqOUxx718sV_VyiF_xTyni
}"#;

/// `aws/NetworkingContentDelivery/PrivateLink`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_PRIVATELINK: &str = r#"sprite $PrivateLink [64x64/16z] {
xPG73kGm24KVOChzJxpA2krueZXJLBRvJKtoFsv0_LbiDfkDcm0oehvXy0lvF48AoCYg2WD6VOY0prFH-WX5QYB0FbkxP9q2UBqFd7VC8KsGyTxmMcU49j_P
lAWp3XeLlJ6t95pSjT6nNouXa8yVplZpybrxxMBHUtj76az-PKAh1CJlvWSQHnW5HYbuDWxwCpwIQxjcXm7COP8c-F5GbiS8raADGoA7mQd6aLwKripvkM4T
ZppKTZxQHJvhtnQo7dy2EViTgmje7othYS1ZVhTf0FKyVexZgyLpzi7Q_rZ_yrK8CYlNbPDWEILK9Z2VUoAtbUlCU2M0-YIqHUcr9atQB9fNkbXrRN-hEsjT
Ow2Uj-OO84k9ymQQjMOejuX5_H5iDflD3m
}"#;

/// `aws/NetworkingContentDelivery/RTBFabric`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_RTBFABRIC: &str = r#"sprite $RTBFabric [64x64/16z] {
xPPL0eH0582mlV-9zzUUODk77KpN_1K4GSE2fjo79JN-yjxxdD8CpWUN_j_mESNxhls0QByBFuttONdGI1-N_eIVACEqdwx3b7zutq_Axsccy_Id_QcXcIhy
ay9Kla-_BpzVVxxz-FQRHlYo__hnu_tZ9mi-Ft-5ppcDzAUqRzUFVltw_GJ10m
}"#;

/// `aws/NetworkingContentDelivery/Route53`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53: &str = r#"sprite $Route53 [64x64/16z] {
xLQ7jYiX21oQxFz_yLk4nEHuvFL-vzOHmK7OiDUVX0zymEoLCM-s7WReOWpmOWhOrMFW1diIfs3qO1nijEuEI8GGSnyIlawnv6u289QyywUGwuA2nDfY7WbH
uovbTMKYLhzsHf2Zkqvr_hQahooiW9xlpcO1v7amRZnbIDfFTvYH7G5Y4I40pC54X0a1JvM7uqU4fo4eyBYN6GKP50B6IK0MPrfzcDbproG-FKFWyGOvB0g-
ZdQDKxZf7ezV9tkPlQc_nYdZL1yEUT9rFrMG7ULAxHCGsWroKcUvuvVu2z-m-ik--H_prymVB2BZfhCJ_Kf3dx_4qGoR_ckVvVy6pYrj_KkXmvSgMzbrzPUN
_XiGKQ___At_f3y-nJgBPAvAA9VH2wZsfxL2zR_1agy7e3--eAX_laI0xayVqoEqhXiuD7ZMxHbRH7Mka9eIBOrLnnq4dmsK64B4y_fuVH9d-pYewzq6RGy3
_YxjbAJl_aiQHZyUCycc-w1dzxGAv1ZxqfoUkpadQ4w8qKo__BmdiLUJ2UKyw2QW4yXpNg9sJuoYwclJAG2iV6OCkfdHzArOXhDxlMAiiKq0nSzQoPUJ1aQP
l3yPsUFw2CbwrFGayVN6vK8rGQylWHllzyTxKNiPbQjpSt27RALicc-4v4lOjsCGZUkFnGS-y0a
}"#;

/// `aws/NetworkingContentDelivery/Route53HostedZone`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53HOSTEDZONE: &str = r#"sprite $Route53HostedZone [64x64/16z] {
tLQ7iXmn25kAGV__mw-oSLRa2kc9fXvZ8I4lSpx-8wXmHL9k7kh85AOZkUEkC0AW1WaWhke0rhVfZdVOHVqT8RVFS6S0eLMm3svYdzUiCzBf0a0_ECZbvWlb
FG7Wev8upsJd9-3lRG8RLfutrhlo0g9dTzRaKmVfNwpvMP4wt3BqgoUXI-PAKeR6cNJqiFZd02L8TYPm82cx6NAJ37Dz-tfafVZ60Tfq0aui09svN-mdPlBI
lj_HzAixiebzxItzQVv2DzijtKRyC3yx_yVywoG_1uHRJhvWRMO5oEDv0DdaRRRzDf9d01Z9C_oIW9ByP8DuU-SRNCixDx276SZpmbQhEO2OhWuluNprUOdw
waoNwCVvyjty7y_l0udyeVlpuG1C4Y1bmYqEFGP-u0yqXWuaAVF4yFyBqMu3gS-n4-WtxfSNuase9z0NPhzSS_aj6-A1_Ohja8L2M7TTNFR8zW9RetUEPkvs
Zz2p4CHzCcE4L2urqG0bwiUgHeLf1sBFOA2lQep0Edu8UlpbU0K
}"#;

/// `aws/NetworkingContentDelivery/Route53ReadinessChecks`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53READINESSCHECKS: &str = r#"sprite $Route53ReadinessChecks [64x64/16z] {
pPS7hkGW44HDntV_2sz2iyYbGVvi_AKqO_EQwWAdwtDhZ8F1KJhQmRHYnGPtu5x004mFuU-j0k2AWyaSyBQfXW9M3m540h6y0mNz1Y_8MRxSXdaooG387uTg
6amefLy7Hx7KFvnRirHZEatD3deQjthHFsr7lsYRRRHcQBUOpCDztBBr2BVDS2cubzgTMWqTAA7qpxDqkPQPwrWDBllsH8CLMPuQqCWAuWoFUsHzsC4Gnmvv
k7ocxPpmqDeFThJzN1iYdhdHzffDuu9wnc5o1PUegMVybNBHry7yWc_u_31-9UhV89xn_MebZ6Ty_IFrT3tZ-_sUO0UuxdU3Fi7LG84zufftwPybVy1bof8q
cdZ0RucR330WDhX4Bcil1GJVxRXUuKk13IxO6g855DS6vDQX1RRuxIPYu6_RGt2HWmal1GJVy4DvMTO7__jRnx0TklAJk-69e5v6y2DIgLzzVb-_yGBHi4mN
z963qVTBlZZAm5duAj3sCTfwuXedT8n7vz40SQb60j3fOyjwDW2vXaY5gV8tFHUWjCgocKgx7sUgqnxZYOBrihrUoIkKtJVQiyAY-t2BMpAl_rpTd6aKqBq-
YOvbZMohKmN4xtw_Vcvz1G
}"#;

/// `aws/NetworkingContentDelivery/Route53Resolver`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVER: &str = r#"sprite $Route53Resolver [64x64/16z] {
pPS7ZkLW28EN_jt_mbjUGtdyQBHzK8s3YI4uvSld3f6VIFPyppRGzuhf2zs0Qm45q07PMIhqkQPV2nW0nwOA4kWU7F00lBK32NMW9yIzV9gAMd1dEa3SeBmB
3CX1Vu2ITDHDkHOTANUbCMTNT-6QJ_PBjjUJhVSs2cnCLVl8t4mkipLpqoMz2-wnoRr7DxJ77DSuM6ExNItcvvtHBVVUBh5jgxN-MWm5qovpP0aZgnUdv50P
ybfDZdxAXFEvUhkVSLAutPq0VKz-L_-YvNo8G8r-T23ogkHZ_Mmqv4DYNosMsXsGWozRxOGvhHmQ7omVQdFkonpoqwDAP-_yuoE5C0kcj-DqjtmalKBdzkcS
rdn4940npicxMXdjouqbgjAxlkzxWYxsRdlj-pv77D09KgVoCYxVdW-eQtrkk4ztz45xVdZMePy5uv89idb-ucFY2h5vViS3jE_zvlaTxuqhQlNyZZSDOlVy
ZWTqoSNpEzwY56pt_8vR-zkcvrln6M--WAsUt_4UAiGmVNU_uW67fw5MV_bqd738qwpkdk-O-63zwlkxfqFzskT36x1-Ie7KrM39Jts06Ktl0cFeIsWzwEC9
KouVAlS2y7aY0AWmMRaM1K1qogm0f1rwJg3PIsjHpoIzujoZ6jz2FVjZ-uuy01_SjT6wLsXssSUqgJWUpjjLxo49L07vS-vOFVlpuU2_-__wkUCR
}"#;

/// `aws/NetworkingContentDelivery/Route53ResolverDNSFirewall`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVERDNSFIREWALL: &str = r#"sprite $Route53ResolverDNSFirewall [64x64/16z] {
rPU7ckqW28GF8F2__mlVwcuq4x_lzi9MbJOmv9ZNlorctwuR7R85SL0JSI357CQ-bu0_sJT0-qtrvjGRe4tD2wg1Yeby08G3ZAaV1NH2Ytb2pOXqPRt7iRws
PZObG5ynVLfoISoplhRI9jGXjREPTsH3T_fodDtG5MywjnhQ3FRD4bSyoUURP43ljVjEgSK6POH13Uto3dbFHWeAvUVuaEjYA1ysfYd-shbhIkU6Ks61MpzI
4PGLpwLDY7iqHSXdBuyEe1y7NRUi8OK-2clHighn-SzKQdMTJ-NgbPp4kqi315ehMlRUvtjAGx95QvAm70DSf6Xu07ksB_3r0J30DVuPqW1pW6lSCnbIh9FB
0D-6ekcDYGNAjh4k_BFIMFG1_CoM0jlYVqvRXccTRY7xQyElmvXB0hNyh-i67UOqI8dtj7H8GOUzcbftuZJ-1IRskRVOe0_JRygkf2V5LaYXuPLj4z8qaKU0
u1Lod0u6X4opa5CfhZ2RFWw776Ww1kCRxAUUiimXrAzAWEjQxNFJE_jriLSoIhqa_ePBXq7JKfwWmuWfUahm7FW-z7DJVHhK2N1omnJBXavWGQg37lPctMA-
FHxjS7EbA-UrZn1pR4DN9YbSaZVinTuDRrLwHJP0PxmqE2xbW9I-gtYYsLwtr1f-jlLH03o_0KHDjFtamwOjb5YhJe_7xEHMg6AHlTNykfHddB1fZIuFbdnZ
RJNXCPib1Vi6yGAwzvUYxypX-sCBYVDtld_-__8H
}"#;

/// `aws/NetworkingContentDelivery/Route53ResolverQueryLogging`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53RESOLVERQUERYLOGGING: &str = r#"sprite $Route53ResolverQueryLogging [64x64/16z] {
pPU55kim34DZrl__yAE1KdSzJHxhSEPha5SQ_xT4nheyUxG1rZEcTux1q35ByV78EibWsEa4sce07uSg805mXW3W3Y3OEu04QfoLG3pIbwce1G3u60u0OIhg
zMWmW1nTpqD9KFMZSorGASzItRjtKnel-ksp5nuAVOnHtKCzcABcfWUFvCSH0vghGkFln7RbOBsFBSMyXAIXgwA7z8xZNqZ797fe06GsuqezojemDFT2UYWV
yp0-ywuVv3FlJvpPX3pZgEEvDCiCY1q-Uam1EppBSnvTvutUSo2VuiHEaeaFsiMESk8L9-d49p2QQellu_VwGqyQTtoi-nV0-KTRvxs5qD4rNe3ewyknoFF_
JYL_kzy__-z4MFvcVsn0M_ajdXNPpTB-MRAxiUnVTrnPVeNdnfMxt-SLy5VxDVPvOcn_RVySe3MoiZ_A0U6biyyReEHtzqzUo6_un_xftnuVGBiRxl50lWVn
VTuvDlh_F-o3Wdnx0jhFFR_fUW8bE-yFKaVJ4s0wqGi96-RLomHbOqbM0AlJJg1Sxc3n0hWmIW80K66NETJY1unWIClmKDg7hXCF5UcJDEPNPKhViEkuaNg2
R-zyWqVUiRtPKj8SwLuq8yO86hOdVehiqvnxzc5V3lYlVh_-t_eA
}"#;

/// `aws/NetworkingContentDelivery/Route53RouteTable`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53ROUTETABLE: &str = r#"sprite $Route53RouteTable [64x64/16z] {
tPLNkkCw3CRmanJ9_M_ubiYEq_sDpdGyk0fnH_Zro1VHr7iP0TYwRvi_V85n-CO2_knB4e1nXsdVVHttv8yEgAfEeh85e6m7LQO1ZAe48BFYmNVFcFiSNQP0
U3biEr136M25-kdynnpM09HpdRUuuBKDYW5bhttFkOchhyvvsf4mnZOo_RLVlayFN_CI_Dn73Tkeu_pd6MC5UNB-COvhqEQNCmA35YtitVrp80t8gafoNxUg
t0yOg4LqE_ZtU__25eZ3N-FhlW2mMCJuBlZNZc-_ld-k1UBz-wUFgw3d_iNHBwygBeoxVYbRR27p1zyZO-uBR0z0HFbbosHuYzD-wKs_Zftd_T9TlqQEmp_f
L_e-FNnJ1eT_tgzusI_Ep_y87OZ7VatVFjs_Cg2wkwZOVEiiQ9k-Mepwjdw9Pxvh_uOlCBvJ__JL_iaM4T-5o4NYT-cViO1_Lz_IOTuKonhRa3HWT0aWgdnk
hh87txC298t0JNFWo7A8P1Ao1b3BwdF_ydN_r6Vzizf_vV0zd8aNsyXwrx_BT8pR_eLVSsnd_Mm1Ng_xj_z5l3z_Jq2bMxziyoEFgsYzzCk1hCn1xkjHKQ9D
dPcc5ZLoyEXBWEtVin0m9-O20xjiDy3j-xv_EHR8xzM_MjMBYHRXB-1V
}"#;

/// `aws/NetworkingContentDelivery/Route53RoutingControls`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ROUTE53ROUTINGCONTROLS: &str = r#"sprite $Route53RoutingControls [64x64/16z] {
pPS7rfmy34Mnwjt_XlyIUwBZqHXEUkvN4KWyoMyerz_D6DzmiCNxqGhoNZ7iaAx0Na00f8NqirIeVPM-5r1OPsaa150z660EM6a70lBg1FWi7of3r9cP1k1V
GZ45EaHBNy4H5Fb5kIH5ZAdK--no2fTWadOjB9c4p3OIj4rLQajDTMopLJMLm-5Ly84kTz4RwlHnjOq7-dALcEkRgPRBjz_0e4RctoQWZooMHmirr_4qloAf
3ifoVgD-LwpRmS8YRaulx-ri6pOkkJ5L6vX1-2B0NYBz8z4g3pPUAi-CfgZd6sVwUaODPrKFE9Yt-uNA5uZNts_8zz82gUgiHUfu4oQWs-K0wiBPi5PvzLoR
w_9NQ92n8kFTcwErAZBU3ROiNNuloX15Iuy4lFnU26sWtPvhCF91GF18DfjMmy8ZrNWXTNhy7Ye_VL82vFdgdQNVjqj0M_ld-TLPRIl-l7x14hsddUINWyhI
iNcqhDyO0U1LmX92zvx6CL-Ov7ao2dBA7q2e9UHZ_W3NDsBLzDhN-BfiqAdPtj7Tt_rwGvGvT8VNFaWjMekJVZcF5T7ZDMnC-Uq1eY8wRm3Zxl5X4MsXdSNX
MJVt2rwq8_qkNTkzEThrTfZxvsp9bZvR9MrS3mnB0Eixfk9q7STSzOJwMFKAYz2heS6YiXlYoSHbZpF9YfyGYthOdf4LiAVSQZHdXSfEVKxhYdqPuLdruGgM
MQz4tumapLRVXe7zwFlhtys_
}"#;

/// `aws/NetworkingContentDelivery/SitetoSiteVPN`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_SITETOSITEVPN: &str = r#"sprite $SitetoSiteVPN [64x64/16z] {
xPD50aKX30CJBDp_mjzTiESoAsomjd6cxEnuIu3Mjzc1kC6M7Hn0fnZb2DWMtJtUKRz10C9Rh_89X7lxIJ2h9DUpxW8MxdD3KtTznJTKzW3TRWd41-yZNYcT
y6dVWqhfjzdqabOzlKX48LHNl-FzE_vVn_XrpPyqQJyFaepX9-Osvii2Lis-S46jleMH34ohJ-XoYLOV7Vokz-_-VztZT-jjoj_hFqLtVqxix1m1
}"#;

/// `aws/NetworkingContentDelivery/TransitGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_TRANSITGATEWAY: &str = r#"sprite $TransitGateway [64x64/16z] {
xLO5ga0n34k5jVU_yFVnIg5F2F8jE5lgbhyI__X7kAa0ekRnd6o20h5unlxWE3xW7x8UI700jZ8D6g6mp6K0_APotSg0o0nWcwHnXowGbTdvcKiWSKqlQhRz
XsJ4jNQjie3AqKiB5DhoO9MnfK_mhW3evlcvyolF-BbEGs2mdFZrowm-PdxzCwkV-TS6IFC-N07rQ16GqNnfFx-mcfQOacUDhYjYQHDWyCbzB_KIsFySY59F
FWL0KUlLeSU3rtg6Jk6h_QA7rGjQ-saA87Q_5ZkgtkETvRFsq_7ZyyVN3r-_VF_m_SlF3tv-yVEJdz_y_k3tLz-VEknFPd_pzmD_l_3t4t-_yVVZpyS__l4A
}"#;

/// `aws/NetworkingContentDelivery/VPCCarrierGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCCARRIERGATEWAY: &str = r#"sprite $VPCCarrierGateway [64x64/16z] {
p9E9UeGm28G54kZxFt6DwVT7SG-FNlJ8BZB3pAZJVovTwXgq-WVbzHo9WHtbTXHTEY2y5fkhL8_UASVHRh9kYlan1erb3F0eMOX3toylPwzIJMJTTdppjJPh
B-6QvXEVlY28sPvaU2QGDlLiksU_X9xcZiAR_cC4PIVBvZ2q_rao2kKc5ybmQssTB1C8ELEHEtshBcrxS35sLZDXQOCTMRAt7CeiY7Jw1PaFy5EHliOJWFNG
LSo2fmoL83ARaIFuRMwsXW22B2Mh-IkeijBfx2INFPwSqK8OV1JkYtmTPh9f1AiSAOeKH2aoMAnOnRGY6lOWA5d31UD38LPyrneng0xZksMEVOQK6BRlFyOL
QX-IAaPuTJfcgQ3tS5fMZ05p7mYpw01K1cg78YC-Jg9gAl95RW1uWk2at-OGnVaZ-4FwCtxa5yVoYv7VxVjt__cv__pI6jvEl3_ttz_Fukk000001a5O_zOF
iQk6483_iF_I_yJ_LduWlvIVo6_r1_GNz2VqD_H7z5VqP_HtxmVl5-ydxpVlH-_N_Gu
}"#;

/// `aws/NetworkingContentDelivery/VPCCustomerGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCCUSTOMERGATEWAY: &str = r#"sprite $VPCCustomerGateway [64x64/16z] {
pDC7ikGW30HGX5BV_yAJfzSboWlyt1LCq9Dp-ylfZtoCUe09tskYWI6XgzgU8CDDRp6Fn2EshaFbk2WQQnrwFig8ZK8UZRFFJswD2I26tVC402_pOuFcd9_n
NkO5zFyqI21bu0N8lUhix57cQO3bk1uBt82K3dH9mAODRBYiI6bttoHB-RjXUIvIrgGzVHis-9oPv4ffUxQSzUgzhVlcvGu0j0rVWG6svodORSDNee3jUWFq
y478hfV3HyJXckT9_nNflbTkEI_Vvfv5maycaEz1p3sB4i7RvmyhCozvA7zx9v-VdmtyvGKmyhbd0mE4Z__EOLhZW6kD0OZd2w0tRCQ-rb-BhmC200081eBz
MmjnjYa0u0VnI_smVpe_Eh-wFxg_U3_qVkdznFkj_mF_b_vF_D_A3yWlo4_8RyYFoA_8pyZlhXzSlxX-Sltc-j7rwzuE
}"#;

/// `aws/NetworkingContentDelivery/VPCElasticNetworkInterface`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCELASTICNETWORKINTERFACE: &str = r#"sprite $VPCElasticNetworkInterface [64x64/16z] {
rPU7SiCm28P1hF_zNxXBBSTP2PMw-zr8OliZ4aQB_ZBZXO-fvaZSpeA8Oy5brzOfX9lA4seUU47tRHUc0ellHHW10Cgqm0e0CTx_SpszcpW0emO3uAqkr3AQ
0044qpjm046ty3LoyzovtZ-fGD0c0UYQ5_2kpm364dBGDWE8Q-kD3h1B3m3iowkWh-4Os4bZ6ZLQ-XYd3P2tO6pcppvRYym6I2a8BgaTPEXX8Yv5b5yFV78W
5SZYDwcRlcIY3E1pdxDa746tVC4LBpu5l7P_zGCBd7v9063tV7kcV5RVa4MZT_oW-g0MFpyp_SU-0EFV-tt-4CyWFzVyzUzlONb_NVsCk6Aqr4zJlxUfzTkC
7wuAFv7VolXfncz6o74acP8Sl-tyCUgYFHTsnYIxVsV-8Z6W-gfllglgeFcrD52hFp_SNWc020YpicnexJI0nIzCVw95MLS3oOO72fP92Av60k1a_KYXhf_f
y_7wcUktFC7Rw_Vdzm-VtxzyVl_q-VtRv_UF6HbZS__w-Vtpv_Vldpy_VFxyyldpq-VFRvy_F_Rdrx_C8m
}"#;

/// `aws/NetworkingContentDelivery/VPCEndpoints`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCENDPOINTS: &str = r#"sprite $VPCEndpoints [64x64/16z] {
pLU7jkiW3FF4-VylViaLvu8ocxTrHaUGBDjGK_cVOL_uEMhMyeqMxqIyBGUqZoK2X6ePlYAogvL_pcwkCa4T6eyALbkWa2CqldJrERY_UChOJAPSG-yNRDuV
gknR3jVqulHiZPUig7Kbi7-WKTIt1u76Nrj37w6Hne7ohBOrVG8jL4IJJS212yFM9Ig0WT6-r_ZEXFR5xCbeST22dGdVbo2-KpA47avIPzt-eUqoM9y8uCmW
jGO-CYWExuDlKn4A6f3qaNQI0QXDV1-qKKtmzmR02U87YiNTTF270PksZX2VjsrFFOXlqoP2DvWlEp0V7HzI_fuV-81Kz9mV5toEMYWby_4Wh_c20b3wpDTn
q9ZFOPMEexghcBkG0FEvQpgwJs01vlUoJOLGFqCi460-Dm1yNouHlyZtJ_XYPmY4F-S_ryz0lw-VSFyYLJoJw7Vzu_s3R22j4zssjx7RFus5ncvobAI1tfEr
uPXiiopST4jEfnM1o2Y_6suQ9EV5IOdYrCuXoIdMawWYDJe-B3KgXNb2XpAXJCleFGiRPZdDgrMJxNDrU1Za433iOSf9AMU70VffRvYO67tPQp27vv7EpmBh
0T1z3cyql_O3TO5uT7RvVcPpUNcWGWsFBJB3-Cx0unfWUKHszdVgdbSKbM3CF_KYkdPxZ2HHH-Ojiz-W-FeJvnUmKj8RI2Kwl3m8YEClHANJII8w2ppVqEE6
3izD7sxt8TTmIfQ1dLmcTz1skAxIhJBb2OwYXJ6v5Wh-g83knufizV8V6BmoAbJUGD5z14u7-JLqXVpN-04
}"#;

/// `aws/NetworkingContentDelivery/VPCInternetGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCINTERNETGATEWAY: &str = r#"sprite $VPCInternetGateway [64x64/16z] {
rPU7ae8X38H98xx_2rlOEMP_KSprkvt625yQKMBxozAXZw6HfwdkxpDY_RIebqquWEpXTfL7nmVfTRgRD98ORDZBjFEP8_5XLruxjm_iGhSWOxCl-mEts5zK
sMuvFCTJsWj9FZD06nynq6iu7CMZscQhINAz1RtMHOwdB7gERiZxzXSSPkMnykAvQt-xpzMH4bypF4VqdclxzxiChjjjAiRfe6-0elrJAOCykZUkpgaRrcgc
IktlzmH4rcJwyU4JiUXRN79482F3XaeeuAaqE5LAKgaQR4lV2jZTLCq21PHcn0XL7BT1VA04KneZT-PRJgUAMX3VAL5oT-1zdWcic5zxUU5r5Yrnkihp9vn-
dMUtIEq3F6uDl7-43_0e_mTuNC1_vpzVl_Rn-_ji_x2XZ_UlV_JxOxogy-oL2r3dENqKGAeyXH-qzZfFFphyMrRdrmV4Pm1LFh2azqlgFBkVL9yhayUy-EHp
hlX11J-qyf0Ud-jr9D0DpsTa7SnInY9kadKUlydWYwEd2yto-EEF1x2K2exaFsX0DmDeVpsX-tO0Zls4Zjd6j-Fnve04Ec0dJ1uXwuu6xMnaYM5F3x0d3N0b
2_W3TLLWW2pH7EjHYSZB4W-udRFe1oH3xI_h0W
}"#;

/// `aws/NetworkingContentDelivery/VPCLattice`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCLATTICE: &str = r#"sprite $VPCLattice [64x64/16z] {
xPO5hkmm3037tFkV-2kfB2rl-kbHPx6Mny72zYauEL5sz3TJQ3fmC0Nwfrb8-Tv-kOZNgj-q4Cf4eylHl5fSP1H80x3itw4BRxV8p99owNQx7-8fi_8xkXdV
Ux6txD8TMm3GIyjxF6oBVdMlkPp1TN_RV6ev_kSivCTya8isG-2et__xzsluft_wGitFXLymFqieCJyNVg1R8ck-OrEuy1MVuILo7soqFoFLNLf0Y2qWsa-m
7luACL8DD0eg56GCmHRr4j1HfHccWzIglXVOGWLGhxqOKejw9KXUn1IyvAB0k_jN2OZd-9UXGDpTlzxSFvqBRCRgsFDBcWg8MjobdCy_V_hy-EVFhn_FoSal
}"#;

/// `aws/NetworkingContentDelivery/VPCNATGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCNATGATEWAY: &str = r#"sprite $VPCNATGateway [64x64/16z] {
pPS9ieCW342DXD3xd_WRTHwCKkBldjawr9SzBDCloxpAOsYs6sBvVqeIR1LBLsdTW69PqoAQhMmhUfss9EsY9BkcOLvDAN0LMHMNEJQk__qNSIVpI7_sq8Ru
7CGtK52My6GAHFohllNxckFlMBRyc0BTtD9n30IgdRp8neSvdEJyJTbND6Wph9MZzvaVUnC8qkc3Wuq5GP5iDMO4QQaU0AMLNnkXuV6rHw2DzRJrKT9T2lmz
1v8hWpzsw8d7KDCGql9vYdbfh6m_uTCqvZ5ADIioiqD4VAA7CkvNYNaXJkE3cDUIwhhLy17BGvxqD0d04UPlnACWxxnGVUPlpDyIhNf8F_Ctu8ihe00ddlbZ
-XQUtvxqj2leyypVxOva1vtiyipVW1Tt09xy7UPFc_bJPE5z1pVoH_sY-QDN7AT-zC_5-KCz_KF_Nfe_SFgN-RaqV-3C3mM8vmUZjCyf0L9CXlFh3E7tzYy2
YFUl_lv9DE7-sTs_NNB0W_JE3rTX4l0GdVDhls8qynUOv_o43yulpi_U-QrEICjRU7vpVwZvIuh8-Fv0uVlrY-ylt8hedqSlOFHlW3z_Vtp-_lhy_Vdv-_lp
xuVdto_Flv-UVx_5xyTdtg-_B7y
}"#;

/// `aws/NetworkingContentDelivery/VPCNetworkAccessControlList`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCNETWORKACCESSCONTROLLIST: &str = r#"sprite $VPCNetworkAccessControlList [64x64/16z] {
pDG7ajqW48HX3KpuxtzXHzL0hRiKd7lZa_Z8SFpF6T_oSpIIJiQxIcRRbPnFjVCjbU7pIpoo-1P_hdFQ_j1cFgjX580tNZ47g77VkBzzsqaWhkeF82_vk1dV
HGK5PSTDhA1qwwBcYtAl-NL9bztIAN0nB_RKsu_pNEk9cKJxcm7riVUnAdVDVNKnFXG2ZZU-GR_nTxv5rwQyHLNOxYMPuCeFlcKexp2t3M7A0rM0yhPj8iX3
U8VegLv-DRgwEPK_Iq0gFtjW0QPyKUSVvQstJ7ujC2XxwgqONv_dDdm7vkQZ2gWw_yJc9-3R142ik-tqWmyPfryb0xh3vrDYyxLbym5m2blJx-10jjzFw5W-
0GVl1LZUh22lVKAP_MR_y_rNywUzdZ-nVjhhzTFxHtkzV_J-rLxlNtr-1aphJ1Zg_EZpg_UlFByz0UzzmjNzDTpt37r_NT-VAzVtvm6yy4qwdukl0m8001W4
OVrRFyIk6a80-07yGduYlzKVq5_Gdz3Vq1_HNz6VqT-z7xnVl9-ytxmVlL_tEm
}"#;

/// `aws/NetworkingContentDelivery/VPCPeeringConnection`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCPEERINGCONNECTION: &str = r#"sprite $VPCPeeringConnection [64x64/16z] {
pLU7qfiX30DFlVyBT_WkFi3XAoaTlx834bF6jAyC2lmULGq9qyz4s51WVCeM1DnK-0T4pH6GSxPnRmCwsva2ULHBCdDAXB3JhnkNwRUKqZqCWBO7WWAmHphF
z5M07WGSy3xJgq1tm7UjspeN6a_8Mk-0pW28udKY8ALR3cg4U2ibAJZEIr__Qy6FBnCw02gIr2fe-xUllTTspc-wZ010E-PNWW1opA-CK3lcLmflNK5cj7KQ
3nkYjm_HXqsqimGnzoUpvI31-q2qK-2S6itkywoF0At4782-QlghHHztY0jHX0IcQQVAwulDmvTeBDKzRGZOP1IP9m3GXHu_Q1o7LW45Nar9qkliTZXDxY8a
eUVq1rtv9oZew_wE8L1Kq1281KW-8w51duEU8VvN9FY1tpEGJc6pR_Yb_r5IdSIr_-_dh_AZ5jVvUxz-L0mHfLJMRxT_372L4GQWxf_j_dLKSDs_gt-Y922h
Wb4MJVxP-xUFsFat9-1Z653Zrtdywu0Tn4_W8NxUn-_xy-F-_Be_F-_Fx-VyeM_pX_lyvJv_ki_VxlF7L0OTvA-N-VDb_dv_VxY_lzpVd-xlR_Vtn_lxw__7
Tm
}"#;

/// `aws/NetworkingContentDelivery/VPCReachabilityAnalyzer`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCREACHABILITYANALYZER: &str = r#"sprite $VPCReachabilityAnalyzer [64x64/16z] {
pPQ7cgCm38HHdNd_9xxFUAlZw1Iuil-M51XRrUZuAOZg3N5ma1UNCBuJL-Lmio2P5umdQVCTnhjNYK-Ha7nPJsAiSaRFINtTV1zoakS7n7FSv5ZHqtmETBwM
mBPwaAGqz2HqzJ_6AgSRLYT0DCZrbYHbs37-ORolYNK19I4Vrw9LRhAkw78S1XAzUitLWyamf46kLUAljOyytgaUON8q0NdSmKcvepTI_wl-fltHYfyuIBZi
ue_M9fFOcE-T8YCoiJbKqN3VtnjTiUwMZKALprwGPRjiTDctWrI_6c1WQRse3K1y2xTD0bMJYeCVvAw_1r4dhDR3dsuEaWYpv_Qhb04dh8fV3SXivA_mFOlw
wPHBNRyrJbg_Vug6BFgtehD9xRxTqUj_rakHulOCrKGimlKnrz8zkYU-1Y1RcsFJtuFiHH2pISL9njg-rZ4-FWtHTQg9NfU9plVvECI7IltOj3ciwEOPgE3l
cPuIssUueQ5N3ZRwWPZ7S_0nuiQb6MOUf9gCYpCKF_UqAqU6aP3D4DicIOQPWnSxFZYvA3-ZDmWvBgCgnq_X5m
}"#;

/// `aws/NetworkingContentDelivery/VPCRouter`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCROUTER: &str = r#"sprite $VPCRouter [64x64/16z] {
pPS7bkGW34Ij17N_2szG9uR23tkrgIOsy8N8ql4_IrE_XaR3Motk6R46KhEhj2FLMxZza4VhIFbrkfcCXMBjcWNjsUmDVsPFqd3NVUV-vRDJxRaQW1Yz0O2n
nm3Gpl6fjI0bayKp0nteCk8zSIxiPxqtilVuvdPxF0iCi6N5QkOx5oYWHvycdGiUyoAJGrqUlFIfEm6WD7UqC-H8Fl-XtiZPE4WldXNJ200SMvu0sgORdWcs
jkSPCS3lyWxOi27aBY_39if_x_7PwQUHtUVjNHKBzstcRJ60U7dI-IZxLzvNRTfw-8d1HzwH_wmduEq86qYUyLGyUSccImF9h_4ibVVqswm0uCad7jGW5-3D
OodL_5dgj_1L_wlpLro_wlwfxj_g-QcUt-BzKRs_glTdzVwkln_rzwl-VlBxpJmNzstySEYBLrexDkjaiaCVl78DV1i_sNZ_sFq0x77jv3TyZtFyc0PIJl7Z
jVZL6TTj01mKZbFyN8lVR-KFPfG_bFANMlvKozzA-MCbV_sVzHq
}"#;

/// `aws/NetworkingContentDelivery/VPCTrafficMirroring`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCTRAFFICMIRRORING: &str = r#"sprite $VPCTrafficMirroring [64x64/16z] {
pPO5aWGX34M9qlU_y4W6FbPiIzd-rO6yM67fFuj3pr3p0tAxvqJ0Dlbb5tg4iflAHrepoNgTTg5-a4f6SaPpZa20coWSPpeFBdTdeM-IblOY-zFWc_j9b-NR
cosEylQYhzK6NppViVCsB5KK4V6nER05rvOM7rMnO0mVkJaGW8sXgpp16Cfb5hpnHnJM7DBEGkOMQ5yIrzWGUJ3WaqyLVEo7MgmuGBuPF4R6y3BmmjK16jxu
bO0ty2aSOC1IurT4O2hWmq4DGQdnWWIuwmQLGGSVROX1zzJuN49Hr_4ZC2nDKWrtnSAZjp5QBJ2CUjUrFVBhV2l08ltP0Bpb3DxRBADfZbR20VX1cb8t0iD6
30wM-1d-47Re93l3mLe_niMvbbrvnQQ0uSBdRffB0mdj3_mGbA2DLtUiZS8TFFE1Ats5Yj2_jZPZsi91ML1Oxf05EUOVq8zC7bOqydANvyWV_OifqwzupxEg
9WdJMZ_i3AV_n8iPVl_Y0HZMZyNVbKz4yr5flxzvN4XMxFVvmmnXX_r3wPo76VOFzc_8NVUyUOoC-nUPuF-H5zKKhQ-LeWtJ-GLU1vwOuqyMyFskGodWfIpm
PVz8nrlOhBdWffjvYGdmmv6lBG7W0my7URu_G57ZixihU1Q9JUpWSNz1WcvW5utz3xZnZl3pao0NsPY_bM5Vxk_Tas3Wpuq5kPtoX6HxsUAWlJ_IkW5NEHoS
FW3zvFru7yVpLvwzNz63msbFqza3EzQeFdo_8yAXj7cx8xkz98UTSZ_86gCPmKyyXDnKHLIjV0PzAhPZaN6w8rRF5SsevgQ8c8dIVzOR
}"#;

/// `aws/NetworkingContentDelivery/VPCVPNConnection`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVPNCONNECTION: &str = r#"sprite $VPCVPNConnection [64x64/16z] {
xTRNSZCn28PXqR_xl-6_93jXHc5jblJo7kj1k7jzweYPnfWD3uNIH0SokymL05n5r090q3Lk08no5O2ltixREEfppkCvCFe-dgzhW7QvL4_uXGK2A9yIwN6g
Ns_0xhXSlxh9WJnNnD2dJQiZTdYgzn00d0VtScm0NdCv3hfNdjqVFEM0WZVA0GKV3M1SprTc6BGj1EyN0BHuHCquUJc0g6H41Iy6E51n0RZaEO0wtf9l0xXr
_xBaMmtVQExreG2qo07J_vrECVIYqyLxyQaFgkFCRlfrDtb7JnO86Nl1Gpxq120SWCsy0t9yhaOUY0F8qFlnUTyyjVS_tjKyyW84im6n9Zu_3ppm-O4CNbEV
VoNcNZ_E__f5j8O-w_jVBtMSsItVwTmRUgspM3hH-aJz1G
}"#;

/// `aws/NetworkingContentDelivery/VPCVPNGateway`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVPNGATEWAY: &str = r#"sprite $VPCVPNGateway [64x64/16z] {
pPS7ikKW30HNeD3tl_36bwWncFMFKy695j8uYH_VMUslhg6HF9JnVq5w8ig-I_jZLk-_vP75N_a-dTrUJrhFlGYj0DpqYZb0jVF9VVEgAe5OnGyWb_Y9osqH
eA3ip20hgCdi4dawRk-qHK65xGHmJIkuARHGLrym6KL551BPW2Q3csHPxVP8aJBR6D8XTCWyU31M0AX9zintI401LqC698VHIW3gPnz6soA13hxaEpHDe0-r
2j-Ml45AhKjU3PPSTtWz5s1ozPGtY5icr9ZOJ6EPHHwLRonaHmP0Z7p5eMha0su0Fl8-_NoQ78hzQruiNlCzeh-1Bu2wp0T_5ND-wj_pnYQi_TFxvy-zvp3X
0_JvsP__UI6fA_KNgU_Flly68U_lZPVFtC3BEmavuHkq2P-WtwzztY3r-xdFEtJv60jl4JRmWeXDw_TF28bcu7Oe1nD0_fzRtwy0d_s_MKX_pje_D9-gIVyW
_SkE2dpIFxsz0Tl7tzu_QcJQXVxrJVtps_ltjwyVthv-qGYKhf_Ujdvxq_hnBUlNxwnV
}"#;

/// `aws/NetworkingContentDelivery/VPCVirtualprivatecloudVPC`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VPCVIRTUALPRIVATECLOUDVPC: &str = r#"sprite $VPCVirtualprivatecloudVPC [64x64/16z] {
xPS5TlD030OZdljV-8VwnGgjhHGUzYicsH4joPTd_Kezgg0ox1Eq5xH2twG1e38o0I3a3Jo1sdsA9S0yX0BkSzP-QuJdKxG-ZY0WByyog0ckgt4ECQkTBlFI
YoMK7xhFWu-AJNQGlHw41cJ2EfMYrh5TEGDUvo27HIhYEcmHfXul4pqhau5EbxYydcGPyVOoKCcfVm72dkZvFWEWz3wafxMrhg9c0VHm1rBFuMWdJbT1HplS
YZwFxZPRdMSk3a06N6Fra3tvEAOrsx7LOwqI1e4tqTOsGHxvXgVgIj0RKS0F-BXrPQ6tGUbWtmJSUQthO9_pdVNz1Eei_guX502fpLyWpdXbWo0sKtN35mpu
aDsL5jpm1dxEQuVQxmtMxu0tG3x5znZyStmRA5Vv1DyfF4jF-PP_ac-fUznBPlnQFydRUxnzadUGHpwf3oCou3Tq0_jFBexDLLkGqBo1BOxDLLjYu3jViBY5
rsrHTHe9tbt0C045GY6zeBeB6RU5ngwIxXNdRQaMxDU6j_jvMwmU4TOdpbW5rSFxyfzAYdwG4Qy1lkXV8-kk-HdBYfP_xg4k2wfkMxTV_GK
}"#;

/// `aws/NetworkingContentDelivery/VerifiedAccess`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VERIFIEDACCESS: &str = r#"sprite $VerifiedAccess [64x64/16z] {
xLQ5hiD037Du___33oibLI4tHWj6zXhqtU5HyS478O34wNBy8KxL2pY28QVgWVWBuhJWP_euKgpW6hyTLqO5mBL-2XnH1WROI_-cTnHZiHFWwr_RMDDv_K4o
z0S4tYcBzxbJFf3m0vByeXr6IqPWGX6rmqYPeanrY2AA9W05pQTt4jMLsDTxqI_U_I3r7PRwSIWMq6FS1YiedDPjW6KsbTEupOO-EOtRpFKMqpULCwuNq8Q-
Yp_-fm-s4NzJFu6b6VJrEuJyhNxH8rsdxT-oskbwgJzkTXLwv0pxd1sBRV3cnBBANaOKeCP-zlIz0H5qRt-yCq07V7j_Wzer5Y16p7cF4zrFJGdWM-TQ31wV
khp_640nxduwtnOZW0-ZS0AeRglWRul7uIpLBOpinlUdwZRtkJ-c8pqgFlZW6m
}"#;

/// `aws/NetworkingContentDelivery/VirtualPrivateCloud`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_VIRTUALPRIVATECLOUD: &str = r#"sprite $VirtualPrivateCloud [64x64/16z] {
xTDLSkL040NHBBxxt_2WsIg_Q0_3-LMaclNt-U-_JWVmRBr36G-iTCg1gFca0l3pFFKWmNGYiD4JOqSpA6WzCt2m1qPT_59fM8nkQhZwvvtIBKvUhibQxT2w
mPYhltxTqWJT04URyzAytHV-TXvmu_UMMYNWDwyslN-CrgBzwyVz50BMkkpnCGQnhAtsNO7i14YfbyN7Dd6mUfp6SesDtdOivldVGXTWT07oBiHxFUFzvi7B
_TFMpzOdW640kVLERJruAKcLmzONjlVIbrxQUndnJNr3V4kl0dBUtwkDvmvxgSBE-zrlqS-BVh23tcaPVTStfi8F-Y2Ln5tlX88yw1lJG7_f2qRGEW2jW0mY
8LMOJYGkEL-5P9IEG4hbODwIOphJaBep3gD31POZJGK7ctVvhdpq7Xq6vZdwY_pttqS
}"#;

/// `aws/NetworkingContentDelivery/all`
const AWS_AWS_NETWORKINGCONTENTDELIVERY_ALL: &str = r#"sprite $APIGateway [64x64/16z] {
xTO5ieOm303XJMd9tl_2pvN9xowBZF7XvhNrxDaSuQA8uPPb0rm4JEasTIXzG7TZ0vYiV4wmwI3c1cssz9e1aMlu10pLskk8DQUX0rtNUUqGtQg0bcxovWHX
4SowsUk0shkTTniF-dTsRk-YX_Lvc9zfU_higXEJFVVm6NGLiXEviyyWKWTX1f6x-azkOsW6iPzFoFry1f4EsYVF_SvVCBGJkT_rIr3tktxV5_-p_NnZFiYR
3_JyFz-_uw3lNq2CpNu4X5NZTmsLNoopPWsrBtMzXjBNkbv3vMjThw7mjQuRzV_hjVNisIi
}"#;

/// `aws/Storage/Backup`
const AWS_AWS_STORAGE_BACKUP: &str = r#"sprite $Backup [64x64/16z] {
xPO55Xin34I_BBh_YKk1UNLZTSluWsiGpaB_hlpdFxLWLNyj2G3PNudXtOxrLmBw_WNx2w8xeFfh200pWFZArWL0V4qJ0yX7yld58JnTmZEOlapA7oYUlExS
IYsyKNOnXBH3iW6MdosvoxbG27vkHd73MrH6o6SxD-frsjxjh8zJ0JdSrengXpPWkIyWjZLNzgkE3ZxjLsnQ2gYE2Ib3X_YJ6pUt0AXtM7Vm88_z8zCWknU0
vY0JhCfWvNp5IEzFSBmzxt_E60VRYJqyL9-9ayOBavqpfVFDN_fVH4yO-T9FuB8_uBZotErkhH_hj5Wj0cHcG7uBl7Vz9VIC2fGwxm9S4XTp1nDOaUd6LfGH
XLZE3KkT1IDxYIfyBKzDHOivWTX6hpHV4fGhHIvjp1ZLXks3Lf1zWGLWlM40TIr-WFpQHo5TekVHkV-mexaVkt-E_wlgfz6w-kIJmwFiRyD__lCM
}"#;

/// `aws/Storage/BackupAWSBackupsupportforAmazonFSxforNetAppONTAP`
const AWS_AWS_STORAGE_BACKUPAWSBACKUPSUPPORTFORAMAZONFSXFORNETAPPONTAP: &str = r#"sprite $BackupAWSBackupsupportforAmazonFSxforNetAppONTAP [64x64/16z] {
xPPNeYmX20KbrlvNV2ClsCY4_pwJTGeKCAnPjsw9HIQGuIPlmy6cT7a1Kbq_qYmg8oBvLJor4F1Z9W7IzPzTJu1uYVUZCmq0VSx7v4WJy8dLuXDoycBp78pu
uHDoZT9nR82iU1eaoUCK1B3roxkxsqC3lRSQWrAi9IbhYHnTIHzzr2V52uHp2hZs20KqHq4fMqyh0uXZHDCzUvzSRIhOQifVC7lH-QN9pdKfvGtl0uYBFLcZ
yXY13P_vwWuP9om9CEELL1y2Ba2QFDd7AA3FgTVpj7HjdrE5ViTT_tFhBblLdYAUv-QQjsuvjaF6T_S1CV0BelCAjlFem0Uvckts7s3aUSa_FlhlWUhppw8Q
3taOJCBEA-I9L_13_HZ4RjR13do2d9AH41k_06byREuLhDT_yGgny-903hcGv5THxRM_5Kyfv_t3Bhm05pxjl7-LrDpTYgz5LRn_In_kdtrU2LfyKz-_Ew-G
8s_DVUENWmsy0FACNu0rlfsVCwz0uzlvFVChZhm0iLxXvSHRmqT-MUEbtPzclehW_VlRDbTPfNv_dE-lfRgyU4Jq--lhQlVdz_bCjmRVklK9
}"#;

/// `aws/Storage/BackupAWSBackupsupportforAmazonS3`
const AWS_AWS_STORAGE_BACKUPAWSBACKUPSUPPORTFORAMAZONS3: &str = r#"sprite $BackupAWSBackupsupportforAmazonS3 [64x64/16z] {
tLU7aYiX37kMCVl_V_n2bj6QCKcqD-MAhst1LKBa_lr4CuFP5ttH-d5QRxZj7CTgONVAZfdN7TwYtupGdiiznVDd2DVSOvSC8q8excFXRbu-0aZkVV_6b02n
MMDevz_Eby2hqQtPpS_fpt5mY0ETSDkLQDTT8_outi-_4-5IFVB49ml0c3uxW4zsBjqYLHzxvdOMI8OZfxSz2NAAN3CMj4SrhBFR7jz0eSKLsw_qDzXknIba
jbkdVpzwta8TwbxBxREhOPaYrU3zH9ZaQ6dCL9zelggOsQKAH-9qxdd4ck6JbGU1iRBuinVCFCN8VwCNhpZBNE_3OjYuEFjXLjJ-TGBunLdwanKQWAdc71Qg
MkvoGYGWaz8d3KEnh01Pl2InhKEtN9TWIv-swe_eZsMOdd3HTt_LN-UL21zrgnIcmSTuVHqiJ8MF0Mo-4OYGtYRbTbOs1hIDKFVigdqZJZg7N86F3qU4piaw
HIxPMvGQZpxt5kEoh7lF4cj5hXJuiu6kCSv92F1NQCpi31KI_3Ks3sqD_jeRqolmrzbI5dJuAxRUFYnHWWlmoyrL-DNXwl0JN7s57ZsBjpuTVS8Kmm_l3_lq
hq3t_H8KONgh0p1xnh67ONueB_WAEf2iQmSiF5vz-D7DuDvESmTT_v3z1m
}"#;

/// `aws/Storage/BackupAuditManager`
const AWS_AWS_STORAGE_BACKUPAUDITMANAGER: &str = r#"sprite $BackupAuditManager [64x64/16z] {
pPO7ZdD128D3_Uv_uhyyeQ0H2TkBjoysOqhUvFQxeVeHTG34k-N9XNp_wMcMxtO0MIxU9LUm_pyDzAFwd_0VLF_UrK19QAIyKMv0QW249c1lzkwzl_ws6XH0
b__iwDEcevSXJwgrfOqxRKyyRSOUND2JsgRwG1SGUlFJktWaPBYSDNZnZuONlCzAxaZRk0P4G8rOLANMbun8E6mfKFpA2iW_6L04L05SoxJJvCaNy78cfH-D
zahmXA_dJ-OKb8fEljHPqVaWL4Q2AppvjmI_x6Gt27JEFcw3tuBEV-eLls-VDg4fuqJH_v2-HBHzFt5eFfwIQgMwove2wJPUzFhewUrTc85sx4HDjP6r7lGL
AfIi9dyYoTFEbT30xeC2dzH29Qtk6MJ2PUDgdAgKVbgT1GEeWSlbFQhRuCk51bdHyfuGe3LmhVgVNcW1vSDLjSPCLaqgzuM7sExR3YTWuu37zbiOq1dPueMP
t-__yMD0jjXlOzdheSU2PPSvqNDLo1bDqB5yWr9EoHd3UymJH-g07FTl72Qt0kgckjkKAFbc__Jc0EccQfu0VjIu5j1d5ihBRd_CHZova2vZnWynTqDKPUpO
We0MJ8X2R7hVtkn9DtAJywH41VO42hW5Zsi4mfobWVMWT70TSF2-HTRkymGzWO_gF-x_u_tx-FmcxENziTiAKTc3lmn_0G
}"#;

/// `aws/Storage/BackupBackupPlan`
const AWS_AWS_STORAGE_BACKUPBACKUPPLAN: &str = r#"sprite $BackupBackupPlan [64x64/16z] {
pTPNSZ91303XKUpxdzY182S4OlOFxbVLfuBDwzUaZxhFZKUbtFF1xeBoUg-i1QrV3tzVoX4VjM3FhrfmrvkRDFDQq7lbFM_cjQ3p0gUicTU2VcvB4ffvBMZc
IPmkHkJdt4y1ALkUof___GfKCJX-VUMdvs_kw_hPzjLnhnvlcMvv2IgN9xoeIfrkabC-zmeg9sfoscVrw9dw-0Oq0Pbxm7zCaRadLBwE7NBk0Vqvjv6NxxuK
8xz4tvCRBoWR-3x8GpxWa3VG8rvdNgJn0lROEyWHRo17VhzspyxwPRrlfwToTluIkh_l4s_WtGVEnA-yUGJrf2PU8M_zUcJWcrUq9FZu-PsG-kt7bndu5K18
wIW-zCi1qatLejx70rz9KjLlcVXAF3cL_gvF6sJcAr6HTKhnjI9bz_a1U43iUk6zt7z-AS3Qzm7WspuW0T_rJYu1tVNBJgVphtv_wR36Zxlf3URP-fa-qVer
lG8
}"#;

/// `aws/Storage/BackupBackupRestore`
const AWS_AWS_STORAGE_BACKUPBACKUPRESTORE: &str = r#"sprite $BackupBackupRestore [64x64/16z] {
tLRlTgTA23CGwFk_yJsZ9tOeFnthbxkxsUrV20QQmTd-P01cm2kgCRzE9ErtvFXgYCSbN0owcxbJrVYS7LxQXXyrVAbyxvhe4N144XV4Gnsl7w8eR_Wnb6yt
A0cDFxJRUZhvcH-fzku05MXy1HP0Ag_pWzpMo4ypaCqUAc3_suY_1ZJdjt2Du2r27Rm4BZerHcPcX2-CG0beCnJIXio2l1EWx8o8tBypyKzOIVRMKpgcqvgR
Sq8KlZKJsRus30F6R0AjyBViyO8Ld_fnYkST7sO_qjdhcplfRjXYvUKoW5OGMVdcP0JT9dxs_ZFexemi-X7jHQ01hkVN8ou7uAV-q0IJK_dyTWOiL9YTRoC3
m57-vE-YeJT9ZZpnfxdPx3_Zf5-wK-eA7-8hIsc5BwpuGkKJnXYWcm--vdU4G_qRbGRrtvmY_XYmvEb4eRXvrXyq0yoPqdycw4dvxPyS5B_j7r9rdYHaIUBi
rCc0UQIR7g1Le4jFlp_ijM03a4zc-zMZ2g2TjBHoZx1wVEfc0edArrxakHdbkj90spDtyyjl-VhXXX_Ep4E5x_fDycM9cFWLd5kLRxCjKxlcnt4PD9Zv-3xr
GA7AOSNUR06s_TECKC7bXJRzO_RgSYd_NBt-xVcTmgM-W4yBcAI-As2NTrYkPg0atiJYbeqez3uPINWJrmf9ks3lGIpwaqT-08CjxSiMHmdR16Wl1RO704-l
XYvJ0nHx3G-HXF3jTu0pnix833gsVnN_0G
}"#;

/// `aws/Storage/BackupBackupVault`
const AWS_AWS_STORAGE_BACKUPBACKUPVAULT: &str = r#"sprite $BackupBackupVault [64x64/16z] {
pPS7UW8m24R33u7xtxX5yubO8dNxt8alUuxFGxWJEkfsD-G-xlUnVC4TIFWOTu2lzuaFWExnUTe95TpcazW5oZVuqvmfe5_0eeSKqFeiuW5_yuDuc9G-YL3a
EuQil38qFRZo3uGlaL46QgQjxteApj7Q-RKUyRFrXMZiK3E-xZSbC3FQ-zBvBitaPt34kujooX95fEmJG4L5bSxY-DIdP3QxyRjQWbZNI6IlKpHXxMTcjTIj
eg3peoy1oRftmVodFsFOvQPf_TLHaHHImhqV_HHj7eE4UbzDDpzuvmEvbNAHUpzokT-lbRR-PIwSlZVj7malOj30tEZx7vVPcD3Q2jp8dHbUNKsnX8vo_1TZ
9dJMvS97C9NpBu7pMUwHn2BiujpD_up6ud8HDsUz_cGuMk2UzI_MvplNtq4h_gxrt-4YWMR_oUdq-z_UZvwlRDBGQpyIqAX32XwzlyuVarFU-_F7VX-S37Bw
yqzvMiael-ly5n1OX1dVVNv-_FvmDtZu_XJtDpMBpnepJP-K9bVuOqFuyzg1PUylIf_JMb-YjYWdEB2evVoNh-47
}"#;

/// `aws/Storage/BackupComplianceReporting`
const AWS_AWS_STORAGE_BACKUPCOMPLIANCEREPORTING: &str = r#"sprite $BackupComplianceReporting [64x64/16z] {
xPO7rfj030RLv_udJW6UzSXcWzBRF-wsXayq6VcpKVqUsm3iU_HlMe37IO4xL9pOEFP5YYGlXdxGgDBO5xMiAwCeEN0EN0uWZsUeb9LNZmeX9mc7TL3GPMlw
jPsEzbKkB7ghEPZ8jLsN3egwvQnqNrtSG2vbImJTxDPtoFWi1Vhi9-pnIVwMGVw0gV_cgtk4ktwJhr5SLEXN-y6D-3hVpbXJ4RMp4ViAtu5gGQm07_i6NTrB
jA6lNTiOe3E_wFJ6eKQ-Gsp6ia_yxQ071hwnTX_Q0tJlRtzBIhlcmOTSzCjFcFYnqIK6le9lT77G9z_0pcAxwVtRBD_1Mf_dAqGVy3TTujdlDmxSzE7sBtXr
U-WDrBFVHvcttaVcewzzdlbzxjNi_ol1DfTLAGDVkNAQ_dhWzwHeUgeC_DxhZHRaUFxcfnC_W9I9t_M4Njdd9ndwFV6XCgyt9YE_yUMwP-Ot6bbG6Vh7Np-y
-GbkGvmrpFWgJ0OB6EcDsX2LFuVt
}"#;

/// `aws/Storage/BackupCompute`
const AWS_AWS_STORAGE_BACKUPCOMPUTE: &str = r#"sprite $BackupCompute [64x64/16z] {
xLO7hkim3Ak6nVkV-0yr0O6KKFtoz-2RhaMB6XwFtnlkty9U0B1kqmkDkk-z8kgsWiDp0VTI1yJNlm7uJ-VVrq_KcR-w9v-u4K02g6V_583yeFEJutwkvHzr
RXpRjWHQgnNtdRkEQC-vRlYr8o5OBLtH69slUR_tj93W2rZ-T1wq_JBHOhZcON7OCsEDukgAJuM5Hd9Hh0KwrtnQjpsj0NpnQlrdvZUCzkUubIJvFIAeLTct
3_vbfWbMHTZRCRcEzSpT0wXCee1mTrFsZmAovHJRX5fUFwCz9nksnpSSS3NR-lELduT-wI6oslqrHtfiG6MS8thOvLCTmMDwZ9y7Ey55z_YKIh2i3ylCzSfV
cMaF5bvjxhtueO-806BcqswwxsRz-ZvaN6_pPre-yzgs5ur1VkhpzQ8CBxX4ISY-2bmGeazr2_2-oLoTj-C-6ehFoU7ycuhFqivyMamjeFb3yKLqaTczz6MS
6I93GmjeZCNN-dHyiuELhkSy5v1l-TFxWVo9BbKMy7O1wprYInGv-cvt3Et7fp9GUK41V5GFyXcgla617Fd6M0dcn4Q-Fxs4CCgdy3KtK9tlPLEfhTKo1jYX
tAKsRBmWer0fRImBTKd-Vtm6
}"#;

/// `aws/Storage/BackupGateway`
const AWS_AWS_STORAGE_BACKUPGATEWAY: &str = r#"sprite $BackupGateway [64x64/16z] {
xPQ7aXOn38DnrVi_CIL_TZGi7arkwN22w_vY9VAM3t-stB-7BW2entXZgPztxuX-x61tRk0Hxa1y-Hg0_tB-kN-gh_FhP_Qf1nj80FsQdmQGRsn-CUxNMlxM
viQVxS22hkQAMdGxxkYi-JpmTM8XcDQSQCZcfQ_tzUV1WXTGlfhh1hjYr_F4bfghQuUDfUIYgCBKt3vhie0gLWEmpwMlBvJdxjDfNwt9-kldtBkmMzvRd8Uf
tW7drwrQL8PxP8Ef3FK6r4QCGmJGjN5wf_fUXXAyJoJBkU0oFTKxjepkSGyEt87TxohE7pK_rI-jGx9JtjRHVdDnv2cVx4qrSChpf2akAdbQfHZh2S_WAKAQ
vxU30WZDiquyxoGldeVSb-PjiCzpLRm8dyu4ByBdqe8NuNDhWbVXqxVWHVYCLl0gV1wEu4Nu7071Y_0v0e8NuNC41C_mnGXK-inxrhAd-QM1tye3NF1w0Ljt
PQkPBrjtRfFpuxiuRzfBkfoVvFpUnVbDoTjEyQwTd4dUNrrYACgNyT83rDVxiar7sxLi2DbsxgCt7Bn1HgDphB5izEtmt_K9
}"#;

/// `aws/Storage/BackupLegalHold`
const AWS_AWS_STORAGE_BACKUPLEGALHOLD: &str = r#"sprite $BackupLegalHold [64x64/16z] {
pPK7SYin34FD-kv_uv_rtmbd1p7J2n9N0HHOBDt-Djm_eow0-h2ykQC_ldj7z8STG9ykFYHtYECrm3-h_mt_nLc_3nNG6WPjxvG7q5u0vGt4kxtdp6--BmS7
yEMR7REREdeTzY3RMCguq_R0QhFsnGNJgQshB-WAoc-vfJWvvCc_5Fn4NjWHcH57n8qRG1NJOlT34is9ZkietFcJAV34D-2aGDrBCPYabM-GfpDxrWT0-i7p
l2GcV0UVrmDIYfoFoZUeixisUiekIKzVXT_tu3a7DSYvRt2obU-S68Be3r9DBAPR4y27VC1kcK-U8_zdNWl_YVJIAGbrwAPDiz0PygFtLNzhUlQ_g2Vyfa-O
lKpCDhthuwvIgWHrDzNFlAZLQr86yN3_vKA3dl2YFmqk5rh1o5Ilwvf0A55beVQ5aAhFI_WrWKRxQz2B_OinHnlSioZ39oDOKEAjTFXqZtdNTZssUzgiIWdT
GFdS3lJ5v9x0dFDTrNCOBlRLcnNFA9CS1HhSeXgwmiIYGdgpzbWXix7CsBhijbo_phv1ibpXrjE9HSw351sG0CB8A5xE4QX8bWAUDrf2kY0Xv_PRgyy3J0S-
g_-y_y_Nx_FzKyJhynEt5UQsB_un_0C
}"#;

/// `aws/Storage/BackupRecoveryPointObjective`
const AWS_AWS_STORAGE_BACKUPRECOVERYPOINTOBJECTIVE: &str = r#"sprite $BackupRecoveryPointObjective [64x64/16z] {
rPO9ZYGx24JRsE3xt_WhgqC_b3nFRTfcwDKfmi0pflBdhxRnQrzfPzRUibhneRXsjuptrOgQQqR4NCgavhjgXflvs25U8tiudYAjnxOlvD7KsA76gzpppnxZ
0X7FvSlBoAeCf13Fv77tzORmETOUhPO2ZklnkSBc-kitzjx8iaxoLAhEVifzuHYIyhXlE_vV5lAj4mALHPzGkBOtsS_Rqg9iuLGRSchPF0k79t64DoB6xue8
W-5RnAYjXWV2qPbuxOerGx8Zb8SNyoSCnCHhL-XKZT1ymzwnhWILej9zwAzVUK8uZEJgq2kKKXGkwLMOWE84v2-B-t6bzAsz242Wfk7PHNetb8Ew69sJuK6F
ff_EHtVi3i_wvPxW0SX24i63lY64_bQAjx9U25_gKq6azpcxIjRF8MAzzZCQyfVPmov3UjwQX1wpRbgFwZW1J8XwEeWBuJLzuD4wuNU7K1y0XFBFO_dI1fyL
vmrl2FMkSNZ-0TvB_SW1-cvlwDrcz2KywBEefo_XUIn8Jr_6Elanqldprt5UsZp3Ms_A0w40n6TLbUFyZjI7SPmZoRk_VysTW7V2LzTFZuw0F6HVqTLUIsgu
OC13--6Y3cicHYxYf-xBmIGYlDO-G8DHvrAO05wu-SRY2KMRcVD96mebzQ_D1yFyNvkfgwLoVMYvdNRkhooDeZvNbwB_u_OV
}"#;

/// `aws/Storage/BackupRecoveryTimeObjective`
const AWS_AWS_STORAGE_BACKUPRECOVERYTIMEOBJECTIVE: &str = r#"sprite $BackupRecoveryTimeObjective [64x64/16z] {
pLQ7ZcGX39lq___YAomMBsjTZXsriSgh1gTWU7qsz0lo7jiBo7T6i3h8D_XPPReGLMy8g0gaeKe_Y2_tV5dbiyQFEp5IJuITijn4OrdV4NhQGgyw9znKHDNY
QxJKNTTv03lJ7MZiHbnqfBP3Lgkl7-r5aAKgg_Bbt8-easHL-HJNWyjydrUBviZ_LsWYprMH8pzLkWCitUcdVIe7Fcfp_aR9_BoAiUMOhvwFl2JAGn3OLUEZ
8K6JklO3y54yGvpnCpvePNYhb3U0umSKmWK2Blc0edRT4SKRXFc0xpBAlbx73rGLQlWMtwfi2v5HE_CX87S0DkIE-FG25PZvHcO0By-RFSrvzqJcZx3PmZrx
hq2uptoerrqgubjLasMshkxUvFDCMKbyF7A_Etfsc1zfvR2o5co8d_XjOS2u03GEX59PzDjiwYQSKwu_PE7AuFwZzxWEW7D09-coUMKxv-aa03cNQVfyKVZW
_TzSSq-lhDAEkKxxRpJ_8siQznU3UjBOCYV52h2QPSXHugtI8G2zgCU50N9t_1loPE-31VJDwTc7e7Jzon3rV1mi13eOUGRGh8u49w6U8K-pKg-3aFRBv7ZP
NCd3tQJzqEYiu2Xb6D490RMCbHE_CdqNPaSXypDa_HZMwqzMDW352pTJSyzlHRZJq94SqWorA70Hp8yXV_7wMFm6
}"#;

/// `aws/Storage/BackupVaultLock`
const AWS_AWS_STORAGE_BACKUPVAULTLOCK: &str = r#"sprite $BackupVaultLock [64x64/16z] {
pPS7biCW30LHGRh_ZTV193_o4q_hzDaQO0GkehJlG_mWjEl-C2C0F-xpz9KVGE5pUm3-UPzaG0_vOdx3bC_vrFs0oGb_jxioRR2eZGXS-gAw5npxO_KDJNrI
9VJRcbxi_8eiymVm5Hr5sDojzDL3yR7Q2dzcrJf0BtqXGWd79kQkbR_1-3nw11XUvclqpGU7czQ37-4Tt2uHWp4BAVh4J2rZq-SeaW7K-SVkqZO4zGXN-Naz
mZtkqxY0scUTy-huUxWjjMyUi9WajUy7maj2__6DRzYzBokVcU_z7eTS--Rs2LyJ7wqgFx__25pxvfRxx2wr7zy_49pSpz__cDlmJM-O-pB_OiOe_4WOFlo8
OG0QTGrDLhxCu96yHVGHNHAhc7y1eHvayU57vFpw4-EKwr_knm2IsMJGr_vo_LU7dlkesEWIxBxSVwpNmIvC5sJ8Rlo1_HUG_k3-3n2BgWZ_zlxv-VF3m_0h
pa-DhciS_kQuM_XB8RdWjmNmUvgSaxK_INt9zWhvdY9FSBwIox79p_07
}"#;

/// `aws/Storage/BackupVirtualMachine`
const AWS_AWS_STORAGE_BACKUPVIRTUALMACHINE: &str = r#"sprite $BackupVirtualMachine [64x64/16z] {
nLS7SWGn23FD-l-BKwun9jd2AaLNruD0kEBnxn1Do0LpHIAE-O64_XW_klgZnBVHWYtn0u1rwP5yjPw3cxaY-MF3WRdR1dqi_AyEScJqMthXFrehFHrKUkNV
sXV32VWO3iob-KOl_8TZMGRwCPq2fVr6JvwuowYe_WksU2M-V_vwnky1Dw80k4IN2I02mBmItu24jUa1m3M0K0SGVUtPwzhDmPAGxgodVMtA79hQImvTxJM7
dlP63gAogRtcy32kasLAqRwHWymwfHmFR6ZVj4nTRe6YVJC7XFdBDdS8FpL73F3SNMvu_YtgDpDu1cNvh7ws_yZn8-SFDt-9zSEiNth_uFSlVl_azs_-_E3F
B_xyvCzljdvWwnUYVYBhDxf-fEjNkdvcwtVs_a3VN_Zxqt_V_-Zxwx_Z7G
}"#;

/// `aws/Storage/BackupVirtualMachineMonitor`
const AWS_AWS_STORAGE_BACKUPVIRTUALMACHINEMONITOR: &str = r#"sprite $BackupVirtualMachineMonitor [64x64/16z] {
pPS7kXin28I34EB-9qwP3OwMxtVFA-FkrI14tn_V5kC2UuMT6zRp_3p1N-17gU-PymN-XEztO17Z9d-6txEPt-2lp8pRT0amNsT-_j7Bbgx73VhKMpIRYxV-
o0nR9O3ehWzXZksQIuAO9G3fzMKLUVZVRpPBSmbWUYsziBNjA2Nytt4MqXii3xJm9m4Oef4vpKdRrPORfiWICpBfjBCB34e1vWzPfS165J7TN7PlzBOz2HYv
Ou2j6zR9q5CH9wx7MOKWkbnmqaOgkfpk5LB2HBwBi6qkPRey8TbPIOj9v6pM5-b2yMQlFPQzrWQeVt22ChnTeP0otoeGOvHcDgZsMJCSB94AMjeX5u7X7bms
Ku3D8Kw5wFddVsGyrfYchxEBNB7hv3TqjDxFwik-z_crI0TemHlI0j5BZJSlhyVxTEG1Q3tiFVRKwFz713YGt1SHqo1-jnnXL26mGLS6P0zba3x9hP2zsDqj
kINiESARByFwKI6m7vBBXy1-JYfWNBuaYEjtNsvT8DTlBOckxc4BgmDH_s2wTjaHqB_kz4-leKVRGV-a_Wr_oYxt-ZSS2Vitpmzi4-qCgi7ymlEJxAx_SNww
Fx_DgWaylzsV7qUi630_lZs_lZy_lp-_lt__m63-vzy_xV1Fkt_a-zVlYry
}"#;

/// `aws/Storage/EFS`
const AWS_AWS_STORAGE_EFS: &str = r#"sprite $EFS [64x64/16z] {
xPPNSZOX30Hx16ZkV-8_E6sebS7bR7_z2Zr5HluItBWXxf1hadjQqRRggsYJvupQ5rhHLVZAUK12PEKxXQQwQ_yOCFvneJ_dEqNtVxguewKtWl3cNvBTzp4G
BGHqRtGlaQ3xeMT0-8NatXdG_3827TE6dLucmNiw2Y1M8zfvm084B0Q8z7a0URzvIu1FtE_5sW3gun4qp-cWicMU4A1HHGiqrOThXj1awzFtDBWwkgAlRJrm
jUK3kIOOzyC4P0SRuNjWVdG0w7Q7ZRPrcViknZtnWsuxeNhwcuLh1dhUZybD5b7vsdPtyd9LVYCy8qMlV4ZFwLduZV0AzVhbw_pmw_nydHzBRhUJItBxloR_
fxYW5VzdyN_N7zkhwuVNrY-lhPzUNx-zldvyVVtw-TouyHS
}"#;

/// `aws/Storage/ElasticBlockStore`
const AWS_AWS_STORAGE_ELASTICBLOCKSTORE: &str = r#"sprite $ElasticBlockStore [64x64/16z] {
xPPLUaKX60FpGo3xNt65uuPyxNNcbGnkUWuw7OnKH6xbEvLn6HzU9Noc_G1GUGY4Ip-LAaTguTgN1wAkYO1NpfS3OavtG_0Q0uUUqfiXOPv3dOIf28Va3Wj3
MHrCZOQXrHZe0L3r7DC4xke3RjL_1pJwXYtMw2lEwsTf_Lhz7K_WTx_xtR_B_KK0c_lN0Ae1sl7-huIwV7z45R5CHmdJm5_Upp-_tt_zF_pr__BN_zFV_szF
yFzy0ZgTRm
}"#;

/// `aws/Storage/ElasticBlockStoreAmazonDataLifecycleManager`
const AWS_AWS_STORAGE_ELASTICBLOCKSTOREAMAZONDATALIFECYCLEMANAGER: &str = r#"sprite $ElasticBlockStoreAmazonDataLifecycleManager [64x64/16z] {
pPQ7pZCv24LDFU__n5ki7O66TUhVxbTZvs0CDoFMdosHRsW3q6_mhm80Pf0VuWA-LW24oJlaV--6M2jXA-rLsi7tFm6P49vi-Sj5o-i2r_yQ0DWpMeAUgcTw
3qpAGrm1x6Cx613o93jxL3ZiUNNyZ3_tcW9st96MgvfxHBYxQLlNmodISGrk2ZsSSoahBuUfsZR8i-cM942uNeU8g2SK5RE6Khi7hriBaEC6mLKK6y7dUnIY
x_I5W7LGHOvAe5w_mUijIOWrUP7zU_Ax0WqtqCaRs6wCTRxlzMY33m0rW2X-PbmlEgzqfHG_owHWjmq5GWn09G6vyGRIl2U3tsOnTztWu1Mi8vqlYubfSqVd
-uiW1xyP1x0Tgv1PivoyEBYn9G4--2mUuiO7W0QeNN5yy7da1KZTH-nVbi0R_69l1fYp8SZ9d-v_TLJOagjRf_l7u1LSJHL8LHC7ARvqx3zRlaHM093hp1le
ZT_GNhOhr9cNwNy9j-lpYrfq_qr1cXHVOSCiOLfMJJkV036A0Liv-82UihBbTpvglVXQbtQyIi9eXlohyP4J235wtAW7mlFdhuBU_8TixZ4lnVjEuZPdnlKY
_XzFq2htpJEGbGqkTrgZMgHZcbCFw-S0uQOg8g9g7b3uIaZNTPGbJEL5U4KwIJpfwWFmWudQUr8xSpDJtMZ3aGFUJVd1a6RFP-He-9G-cN0rptZrpX_Erl8y
GLmEz8Xzo94q6VqSXv_jKmvKCy-RWPvAWo6NzOwqEp1StuAx1KNMdwo_0G
}"#;

/// `aws/Storage/ElasticBlockStoreMultipleVolumes`
const AWS_AWS_STORAGE_ELASTICBLOCKSTOREMULTIPLEVOLUMES: &str = r#"sprite $ElasticBlockStoreMultipleVolumes [64x64/16z] {
xTLPGWCX381Xij__nXe6NB3fP73NVA_qRqcwjGzH2l5jaClH0fBAmKAKwTMCekTDJC3XFQcVMAVnyF6P-Cs2NMGksG_P-OpfuTS9dby67RmUtjITlGVKnrnN
WPO0SnVIrbN83Rp2_Z2_8U6ZbnD3XYzfW6NZQWdOP2v86n2fcLxhNw1NA_1PxcFlrGQTlT7ejTs7IzzWkD8JTzwZ7EXAFuZtTk3TNjr-VzZegw---kh1xVTe
xljwe0Fk_dzCNzaJJszwqnFU1zB_to6MXBZVLye3
}"#;

/// `aws/Storage/ElasticBlockStoreSnapshot`
const AWS_AWS_STORAGE_ELASTICBLOCKSTORESNAPSHOT: &str = r#"sprite $ElasticBlockStoreSnapshot [64x64/16z] {
pPPNOYCn34F51jp_nBj9v6uDnFIyJvkW2aYEnX_4V45V7bnHdvBt4pZ9y2T0PbU-EgknaM26XVfdBqUigRapIN73q2Grs5-0P7mp3itEGN33NezZEdeeT_fw
yyYiLpB3_Kali2ZmYOgMUPPtslj4QJjrdMGPVeqrZtFMaXbYQdrT6ZD7Qdcvt9vCqEXF0sd7aoUqMWTPOts7Y259A9i396bAFyrx3Z5FSAAQ9ta6bHclPDQr
7XD2N-IqUGtATOFb-MmUMRXJNkgHB5K_lfY53nvbmzj-Y4vUBDDRFBCyN7BnY_MdLt5sr8D5si3dbm3ye_CnAZ6h-d92c1m0TZIJqgPU6U07JuVaQHEc7p96
XkeyWaor0r5TVXwj8A4Au9WeJaeMW3QuWB9U0VJpw_BKvzC6wWRJ1khkxHkekrSVK5-Flc1jZuxGsNK7zq-dErZVhYu0xQwk49rS0pB4ysJiYFLRpMRlPkS5
_Vpz1ISnEdXn1MAqyNZy_dfFA3w_VvJ_
}"#;

/// `aws/Storage/ElasticBlockStoreVolumegp3`
const AWS_AWS_STORAGE_ELASTICBLOCKSTOREVOLUMEGP3: &str = r#"sprite $ElasticBlockStoreVolumegp3 [64x64/16z] {
xPO5jiD020211UR-D_wFVhUmMxTfunaaBjU6RTBgoIRPw453REe3qZP861kw0Yhxxx0BtnQurNn8Gq0inEvhx7lhT-kRxyiCS9d28VwAIlWKmH-LiiZlefPm
EJBcIrXptxOStm-PngVzf__qd_w8mWxoanLVFsRqROhBjE-GWGYU43_qtZT2BSjFKsVKvhHvFwXzog-af8WbfExd9-z5xUcdx23d-tVHB3z4j1RMyai0EvzC
K8ctwQmlMaxLRpvgSNXDf_s2B7-lwr_90DlR5tFtwdROWpm_dlxJJvicZl_z5mVwOal8jV02
}"#;

/// `aws/Storage/ElasticDisasterRecovery`
const AWS_AWS_STORAGE_ELASTICDISASTERRECOVERY: &str = r#"sprite $ElasticDisasterRecovery [64x64/16z] {
xPM7ZiCm240Vm_J_l_WcAgU6-1AhsdwQ8MA3URoPpMOJfe2ehsahJnOiEA0ZlkouWCHzTNj-385nItsyA2Z4dTmFAIjoGz_GLv7WZt3pY-wZoKYp5pVT9mAO
wHK3qbLx_3PnNAWaj3rfW-lJN_wRfPf7v_EvPzvh1JGcph5vXLo1SQgkhSkaOkmDIAS-AFt4u6JjMgDsg9nYRMtyl1-6_1LAwoaweJTR0HSdpv7AedKrruTH
PZhzkJ2pZZGpZryFRKsz707zPmqSFNaBBFNd_Gk8Jd3etqXvdssXDNPEgVZf4WhYPJd39kfDpejYP-ihK4JcK9kHlyTiFQNqSrJh0CZzY-fO9h-6ufdy6iB8
sLy3Clar8WkvYcUxLZ7Maw_Jlux99FdDPlC9
}"#;

/// `aws/Storage/ElasticFileSystemElasticThroughput`
const AWS_AWS_STORAGE_ELASTICFILESYSTEMELASTICTHROUGHPUT: &str = r#"sprite $ElasticFileSystemElasticThroughput [64x64/16z] {
xPK7qjqW30QjklS_SQhbQ4YSPo0z_qvzPK4I7yDnyOOcHUg2JoFdTO7qJyIgRsSZu0ljmzbrmZ6FWLGXiU1x-G6ovjSvvB50GAsZAtwI5OFmep2_UD7KEJMb
voTeo1F_IA1Q43h-8ZxUHgdjJ1-CjU8hPmlPPjhcUuFgLUfPj7pwLY5wlbm_8AFjVDAiPfHWrtsqLk19ogX3Yv3NfkFCyzkN6zBqIujwhVPLiVg901kkNOyG
B-C7VkD3nEkx3t7ZLtTBVarO7ldcRgEl5Tm7VW0w-5OPN_Izcd_WbpVuikEBwx7ZT_PzJT_o4sJ3Do0s_0IGPTyW89RzX7h87lbQNDlxy92gViTF2cjFU2iW
0EBMvo9g-_wIr_fvxwT-v1GImZzGHn2GPWbnvu-rNNXxUnAU-6qSvt31phRqaT_Pztlzp-RNUJY_OlI7ytkMdv7ApxFyTfxbzna_oPzbt_TUj_aqKlugFzut
zoq_GAVmqP-d-yOySZHqcj9_9-y1
}"#;

/// `aws/Storage/ElasticFileSystemFileSystem`
const AWS_AWS_STORAGE_ELASTICFILESYSTEMFILESYSTEM: &str = r#"sprite $ElasticFileSystemFileSystem [64x64/16z] {
xPE7RYqn20KfXld_Btx5YSnUqfRSg6VKLaSQcJN7ild5GI4Sz6InCm-GGv86bkIF-fgGR49UiSgyZZbOsMibHIph6MlfpdZ8QYWWD-SuMcOYsZEz-hAngvzd
wFBNkPloHOFoNNUaIVdscbrztTdpvp_lkoVbX-lEhluwMVdkjR_oQtzNFyx-oc_xyqqyp5yPU_xAilz_KNvHVd6jkb6yxMgLFJY4N5bW4Akz7uFLtu-3__Jx
4pKJCrMpppCps_7EHdYyC1RxpQtlw4hn7T_1wkDZz0FEOWpiV_2kd7n8-TNIhiF4nzzUSVY7cBnWlbb0yo7aFWzLjVz0sZtqzI9hyY7sfByU_GMkzqdoqJTm
kS-2YxyY4ZBkOhl3npUcYCAgyvdY0b7-0fFxG4f0-JhhtG9pqEDBWCu6CF3o3TR03p28yYLXu2j4aXVVmMS317Zv0Wny0o1Q_-K-2HjxMEfzlFdZ3ZL_tg7c
ZplK_7c7cZ_kKFFd7Mh-k4FDdtUe-VCETNz1t2VflZB5Ugjcf3pBBty1
}"#;

/// `aws/Storage/ElasticFileSystemIntelligentTiering`
const AWS_AWS_STORAGE_ELASTICFILESYSTEMINTELLIGENTTIERING: &str = r#"sprite $ElasticFileSystemIntelligentTiering [64x64/16z] {
tPM73kiW302zx_-_-2rB5XMA6fgthzlfCUm0yd-Yt_dSfj6FP1Md2JaVH887k3caGPxgCtIQFDGBe4IapXi8A3LIEnE9Qq4SJNuA-FrTn4y6O7G33a1q19F3
0IHDdNMl1T1wU1HKPx5Cn3FjJkr254gb2s0zbUgB-jRFL0cm7W-aQERuySwl_a2QefPumIC_hYUWPjh9z_L_a2BkrRv1s7UsgbSNEG5S34gcc0PGwwgcz5M7
zfL9TRbvJTxeBe6cNlo2iBdZyLave69hT_eTIx15LwOkSxS60Bb4Tc5nR5y_aoQ7abSWB_o04flc7PE4lErBGG5a3mOwYRTzgMt_QdtpTVUd6W6k1kcZRtuG
B_vWB_ktoOMV4BlVACqcBxuLNFcYzXsLIxyBQ-CVqRwzzVMP7zGJNm5_u2V0vxu241_x2LRmgM-GuYy3C3lm2xJVrtJafHyqjNG_7xCkgIiVQ6psWSoSGokX
t0loqg-8o8aDdG7c2BhqrvWKWtNSPTRC6t_ZeNzp_6LcSU7VoL-p-_VgLz3efjwwVqIraKlc6hj_m2_m3tdkn-ABnp4o_cVy0jyp7_a9TaHi_Z6hxvoZig3d
o9_dAm
}"#;

/// `aws/Storage/ElasticFileSystemOneZone`
const AWS_AWS_STORAGE_ELASTICFILESYSTEMONEZONE: &str = r#"sprite $ElasticFileSystemOneZone [64x64/16z] {
tPLN0jiW44IrTUv_OXTmDDa4BC5FohXRFDYkus-LpYw8gbpIaeZX_uwca2iSemqSf-827oq90EcgrhvSuTo0fwToIKnv883yDGHQpWnW7c8_W9265qBU7vdj
GjjjAY36Vw6fRxV42s2Dokwx0Rh2QrkVoFRMVw-VJzu0MVMVzb_vxsw3wlafVj8sMP42plor_AyfGOzbddxpKrPyKI3ANcCSeBJ4Ca-RVUvz0bGKNW08Ms7M
F5uYJfx7OX0ySPUZbgNruwhRtjyhgp_gKGCe6PCFB_gI202bynm6yk7BO6chMNZlJqLJmG6GBtl_plVqw1kZJmWxxLUjixhQtsFeBJR6QSN-lD5BPXm_dRC5
lkIMt9pNnkik7p0W0lQTZtX_HWoQjYk_yW5aUyQadLA-yf9SDjeWvS9_oGdUDaYvZB_a67z9NMxaJvBuH7EU-gzuJzdbEIRt-G2W-xpYfTpd4mZ0DdZUhieu
ZKJwb3VUzNYBSnhWwDBf_EZYR-yZc7sOWPVa5f-0TCmvoviLCU7Bfu7w76yq_9ftKFqt_T0R_E7EU2xpLUlsLzsDN_XBCU5NyqVTgvyZPxUAMhz_uVnoNLUI
trNd1LjYc4NtTFmDUW8
}"#;

/// `aws/Storage/ElasticFileSystemOneZoneInfrequentAccess`
const AWS_AWS_STORAGE_ELASTICFILESYSTEMONEZONEINFREQUENTACCESS: &str = r#"sprite $ElasticFileSystemOneZoneInfrequentAccess [64x64/16z] {
xPLLOkKm34KjiEx-LpmWfxVZH2xePwZdyOkFA3J-EaHL-h8ZcRqG3ZB72RDnO088LxNySbuqZzb3Ho9nMZMFnIaGspejzMJMc-JdHqHk1Q8Ua0W07mzML2Gh
N55Zvf2gDW4ihLYzMzc0IgwjjkJwG5nlIh-cpl_g6o3bElVyl-oVzRF_kiwA9I39SkP7VMvmxZ_k_zel2NQSwQFet5zy_uay8iyY5W60vRFDdM1fNdJlJzyU
kn32Kw5eR-__aODfKSq8yDqFH973r_lzcgOEs7xyJ40t93lGU_CIM4Wn4B9gLYJtgV1VaS5tBVnXBr_armlMLTuGX6H1Ay0DkQhoZ6FRTO4bXmZdKsAsNNvq
syqylyvm8NqjhfyzdzVllZ_SHy_V-VA_V0zf-xnCzlq0O7tV023wFW07hEkd2u1_o0o47ttp2-CTCdjmnbjexIk8veBr6O1m6Z41GDxrURFb9-zP3ipQtpy3
8BBI6ml_W0-oH0UuL_K3llehe-5lDEhV-E3yflzW5dvZ_sryvFZX7aDKDe_VF_3wvVfv0lIxB5_GWsCMxJ7-1Bu3
}"#;

/// `aws/Storage/ElasticFileSystemStandard`
const AWS_AWS_STORAGE_ELASTICFILESYSTEMSTANDARD: &str = r#"sprite $ElasticFileSystemStandard [64x64/16z] {
vTO7pkOW30PXR6pc_YVUYlG6vs-GxJjVBuzafRS_DxD7l8C_u-3FEFWp3dxi4nBwwG3ku5ryEzVm_ucFw7SVtslD8knj3tdpoVSqpvOt3dtTYGvzGFk0DtUR
tlpQgDxThMeVW4-Vh8tL0mnRERpXu-uPYn-GTNxLRlFdkY0CeYo_chn0_TwYGLg-edlN_loIDqWOeuvWzNYYELS_-5Rn7URhJBxEZxY2EMiNdykv-6OhbpAz
xLhEGTJbUkFjEbAVNDj9hBwP5nxG1vZEP_8-eFYQmMYEffBNhlc6J_3vNBtSwzvWw6NV9upu4k0d7fKdFkNxnxo_STRzY4_DU-hZIuitjnslttLef9zvBP0u
z43s0FoZthzqtUL2E-R-yW_GN1_mEagT-UQpTZQ_Ebj-wcpzgRFjH-rklnlzUB_RSn_NkNjiPt_UzN6FUEGJVAkeVhSoF_jPk-JRYV--FWC
}"#;

/// `aws/Storage/ElasticFileSystemStandardInfrequentAccess`
const AWS_AWS_STORAGE_ELASTICFILESYSTEMSTANDARDINFREQUENTACCESS: &str = r#"sprite $ElasticFileSystemStandardInfrequentAccess [64x64/16z] {
xTO7LiCm401HRTFS_yGqnU2t9C9hqM7evITnKMdVBv4bRc1h76oDWormCy1EUriw1MRRvwzAL6N5A-ZxUUKbF-7PfsMl47u9ekmTf5qAAFbi3EGw4VF7ylLp
T-YlZqKITqOj1Uozi6NvdX_b-pV_Rx2LdqOBuoV_LS1gunSydU_Q_06H_6J-glfnU4LVch-7Lyi-D_TEbpUywyG7nDm7J7mBa9bNm2TUGMU-0yXfhz0XJliE
4Y0dlO4tWJZizTBsFunph0UziPySV7mLw0TzC1glc3nfMwUxpBq-175brP0E03hrkx5b_jnsBqG7v93FwVYjWPTyNaKwVURzeLj-6r67pd_EG0ViDxrE5ba1
j4c0tVBpd55lvtppmUMiR-BkMXX_axwVTosLVRsz5-hPpZVHSilxrFVt0QlUIqNszVPUATVRV_S
}"#;

/// `aws/Storage/FSx`
const AWS_AWS_STORAGE_FSX: &str = r#"sprite $FSx [64x64/16z] {
xTC5RiH0343HRxx_YKjULKvACrL5rRnb-0xNtpkEupWoifR9y9g5y4fiSGP3LYcekQd0OYyr65ej_HTzNZxjzu7LfFB-EKJonzxncWAY6jZ-yiiWUqsGfUqd
wqBG7amkzEKGDINuutcbBmVnsnUz5by0QOyi5T2n4khbIAtqBLr0uh8DiN3yXz1vk0IhzRu5-3o8ktqbF2Pb2RhTbv0zHyVXs-vxunrojuVUTkafijaxCixR
15leCsudCjZul73zekRX9k3NgqwXFYDQWolDicwx3RHMP7Xa_LF7SHpF
}"#;

/// `aws/Storage/FSxforLustre`
const AWS_AWS_STORAGE_FSXFORLUSTRE: &str = r#"sprite $FSxforLustre [64x64/16z] {
xT954WCn34JH5lxxdpYW2lEiupSCxJA8VxSioxBuIKEQPD5i8e_Hc6LLXhE92aOQJxRaFJWpvwXKM_BEMIc1bc_g_-KEGWskJVqd5Cs9sRJt0ycE6YYN9QEp
5HobsySrLTOHeqEgIm-0qYYyllQVoz2FN0qfMNObRloOnmAeQ3abnQMnJNa5vTWbdaJyc5V2TBTBq-jo1ILzoTTqiSqkfH3AEKypflgS3ydSv4rVkbqJL9p6
y4fpKPdPqt56Dq1ASst-tR8iox87
}"#;

/// `aws/Storage/FSxforNetAppONTAP`
const AWS_AWS_STORAGE_FSXFORNETAPPONTAP: &str = r#"sprite $FSxforNetAppONTAP [64x64/16z] {
xT8tTkH8403HI1dtl_6-nYEupgXgEYFm_IYP_k4UZyVZaHzMEo1gFKBMPOtecQxq2AEN3czlzLdU8dqPvgHFRqEhInwTd_5Mh2QUz1QrVeYm6Ny1nCkgaqHx
sLxANq7FYvSjcCyJlEpChvt_LgmloUecG9tqHgbJIxf01kHnJpbfNkL2HFroJsk6kWk-343-h9zLtOZFKUWxgXoOd_G5aK6kIrqvbBVytWdMJzFTg-wexl_5
hZQq9ugynm3K8GxxLg4--WLFDA428zolUIAuKX0qOJooGGj1WGRjgLrX0NHz2zY0xmADB_GHkk2NxFhNU3mU_m4
}"#;

/// `aws/Storage/FSxforOpenZFS`
const AWS_AWS_STORAGE_FSXFOROPENZFS: &str = r#"sprite $FSxforOpenZFS [64x64/16z] {
xT8tWWCX40NHQlllVs9P5Arj58gNZAqnW3xUSHn7BYrLc4TfXYNbcBlXmpxqbAHkkZPxCotfgRlm6lIuNenxMBW6lTDQ6fC4FUaRuXKaBgVq3s1t9RM3hyOY
2Vq7vZVFhWEiTTEDpSP_IT1TRVOgKbAK4nlzEb64ZDv-FsoitzPD6v9Jyzx0tA2qfexbkQSVP6kfSFUKbAdl-xc6qBu6rnlicRFCSzCRlKDhN-pdW_cRzCTn
7CSL
}"#;

/// `aws/Storage/FSxforWFS`
const AWS_AWS_STORAGE_FSXFORWFS: &str = r#"sprite $FSxforWFS [64x64/16z] {
xT45TW0X40FGX97S_yGrvYrKL-ej_maQLB_SiYpBObPIoaQ0bZekmH8AqDoDE9MFKMrLkPPtJDsHet8mRvoxgq4SpqE3mtgVdxY_QI2YdkHeNWR6uniGsiqt
Uo19GT1Ld3B4uV_lpMWOjuX-UgApdVDtQM3ERvyetC-FRKSbUp0FvsNeLr63NypNVF1Alb5IGqf1YK2ne_StoU6Iv4fi-7RUcaTcs7HeJRNSosSRnoHI1Pkc
FHbHkS5HcFfcox8iooq
}"#;

/// `aws/Storage/FileCache`
const AWS_AWS_STORAGE_FILECACHE: &str = r#"sprite $FileCache [64x64/16z] {
xTO5OiGm30NH-R8itV_4nMLoh9IRMOONflmHRMqPnrJmdAQApqD2gtoAUzd03mSmy8C-rnkBidkUXTttkQ1VvQLLNg0Ltf2mikyG0RtgHSjqLFGxARpcTz1H
oSV-8qJ5-tx96rxmH-POnOl8TCyCLF6GwU2PKF1nwcFU2yltwKz3i_xS29lq_MoQ0_gST-nYNSwEN-cn6xkdPiSVqFOVeK7C-GIWxnO5a9DU87O9DFQzd_xY
z61NT7pas_Wa-_ZdjjXRuRAY6gookySVypzW-V-0jhQU0G
}"#;

/// `aws/Storage/FileCacheHybridNFSlinkeddatasets`
const AWS_AWS_STORAGE_FILECACHEHYBRIDNFSLINKEDDATASETS: &str = r#"sprite $FileCacheHybridNFSlinkeddatasets [64x64/16z] {
xPK7aWKX24HfGBp_YJUuyckM4cLpx0hzmxm1KqlVKbmICbnwJKZuLkGvv_aetV9q59_uV_wVnzjuT7qJRxqRNiwBph8mFsK5FZtAMkFpB35KUD6bp9FYm7FV
wOsybdYc85y2eTVod4nVaTS7l2tW8f_qNkObwRtAKrAypDFdyAwlut6vq8T7oOQ7NzWsVfaUSg2bF8U3uXuE0H8UZuM5cQKzRJVtc249tsQrsObxcc6Hxzsk
bVZc-mnr7jxh_ABkvrZXP-zH1P4luetN7HzBnZwy_Quh2SrItVqwPLp849ypW_iDiNs6nPGnJ6-gOpTYdQ6gihwaUBWhbbD_uteet8od3reMgwsKy3M1tiRJ
aGUAV9udFvh_vvMFsl4LvJoAl50YS4dqhNGF
}"#;

/// `aws/Storage/FileCacheOnpremisesNFSlinkeddatasets`
const AWS_AWS_STORAGE_FILECACHEONPREMISESNFSLINKEDDATASETS: &str = r#"sprite $FileCacheOnpremisesNFSlinkeddatasets [64x64/16z] {
xPS7SiD024NftF_4AOnHGPaDf1V_ATysUjR2TlX1GXe92yu-bC1Hw6FXaITteO74dGfFC13zK1v5C7tAL_zolkYlynyv_i6ckDK9BEgkqb0ynukSMhb_D3zl
_tlhzp7zTn__tVKY_Qlvkgv-TFwfx-FniAEd0pRtJzIS4V5BEcXl_5Cv4uGtUDaddZ4HyVFp390Ek6YxR5bg_DFMalHsxOuB0KSWlSLZj7N_8kclvM_umhwV
dPZyKfTNbdQmBAIPWxjldifWfmOSgQI7nDtwuzUY63TNaKWR-tpKYS9BRPiya1hTF6Z-NVC_9pl-ylr3PQ3B4FMnoWPXCzfmaLD-RiG-bfVZ-jw8LVueQiHM
PoXin1PNM6p4lbkF
}"#;

/// `aws/Storage/FileCacheS3linkeddatasets`
const AWS_AWS_STORAGE_FILECACHES3LINKEDDATASETS: &str = r#"sprite $FileCacheS3linkeddatasets [64x64/16z] {
pLS7ikmm20jLzp_nQrvc27dHZt_N5BPHZ0Jn7lylv0EgyeazF_F8n1aPOIeFN1CCOKgSiwLpDzClc9b790hnRGYDbwVVfH2rm88T4-iRrn-USCN8ymzbriL3
N4f887T8SW0jcm5-R461F8KxDX70AN_yIdwzSa8XWLD6AN6D7bAPzOwXa5JbebugEzURoCCIKxw58M2uAA3NRlRfuHDaG9Kv29IHgzTKPvkCLxWEaRO_XD6_
2DHZ1uaSysDR_bDmcmCWy_T0RFeRwmT7xzV-11PVx8E2H5ITD-auWJMNX7v6e050CFevWTfNBQVhBaz58FJtYYRqjUvjrcLxomhmne_joLryFFVysbPHrPhk
iWsMlOWLJsQ9rQcotprVq-IB_cm158NLrx8tHQG1UJYGnGQGfgeMIsvb2xE8zVi4jCJehP9cIOZM0I0xCrgMJD-23-cNiieIVnNLCNhYd-mKYSpVL-KavSZF
fnT0sVuJVRgy3KXfTbMOpvTtuijpsN2FnGtnzyk1e5-saDETxogDr9UjSTzOtfkmBNzEVuUgx5tTE2sC_etFIHvQ5wMVz8M1Bs_54sBiNfjU-v7VFU5yiQIR
iFzMHkBxwGQg2-OUsHOqXP8xQFepuQfvJDUmRT6BYAhMzVF_n3i
}"#;

/// `aws/Storage/S3onOutposts`
const AWS_AWS_STORAGE_S3ONOUTPOSTS: &str = r#"sprite $S3onOutposts [64x64/16z] {
xPO5akKm30LRJ-3xdtXXoECVqZ9CkZXMgmIXyNTmS-DiyE_GlpS1v7WWeU_lnVvh_kqp-Jd-FR_C28_8hE_mnNiKzGtrvvECoRyYuokghk-0bg8h14JFp_r8
XshqFqQWiOSHBJy1hx5I3cIh_k89yuZ8p8Xmuubg-GFIZHNpXD7rdyOOBiaaUUHJMTRqRRVGH4rVSr0-FoI1DtqdvXaMyshF3xH-swAUglAcdykamW3bC537
hn3iFKw0hltWYLXpkl74fH-W2Cq4Izwuz25UeWtV6UgvFwC4B0aSR_XwBR6K1RMSNFk-FAFBneoyz7qjkHmqe-hGpwEOUd_M-VyQHuz2OUC8F_Hdruwklo5w
30gegdG1UUqNXwWwtz3IZcho77yLDpUV0G
}"#;

/// `aws/Storage/SimpleStorageService`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICE: &str = r#"sprite $SimpleStorageService [64x64/16z] {
xPL5ckKm38H_OUv_uY5Dax-adBTghAqi6GfyV3Jyu0UTuMuCp3so7-Wrpc2tPaopUMJrhE-k3BSPiUi6Y3gjLG382GPUebuETbrsh0wDCloo2Br_NH3zlkrt
KDUttB9ZQZRW5pntEg3E6CJGZqTMzLDZLSRqMWOXsFVPvXHpC1SAgAUxZUACWOqQfsIKvaP3StohVpDSemLz9vUpTNb9YQsFcx3GbOJe3zRLB8uCQU1SJsSO
l7Qb8_rzzZflOiEmUjYSQS_N40PGx9Op0Ge9rjS4NfKw00mxsh1h0akd7-0YGZGzan28297ocFbTkPnHoLFVJzj3lwycK9UrWxM-D6ohRmr_p5zQ0DhVMW3w
T5e0-dNs0j3iQC2Tejv2qw_XkAZ5jh-1xBD20bEGQ0FCb__RZIMtPCwSmUkuZVd06JnztnyFF_Z1Rm
}"#;

/// `aws/Storage/SimpleStorageServiceBucket`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICEBUCKET: &str = r#"sprite $SimpleStorageServiceBucket [64x64/16z] {
pLU7Skn027kY_lyVls6kO2wHXysqK-ewmCeIM_BlMu62200S9efvH5uHuIROvAedHzXj3UdSS3DLAQYgUTDn8TcQ3p2cygPcDDiLgzVJpp_uUBlx4b6nys5q
pGKqFWfOfXzPx5E46EKe9k7BA8_pVUJ7AN-RCteUwvS-onLC5ido2fxDu-_ckyrzXKswKaEDI_Eh0EY5BDSb45e9rtcGQMFnSRZYGNZqQik1nqm3pGejMFsW
3LICLUq1LHM0sD_Z1UXyTfW_x6yzDcYz6-t713ozwqw_P-3lxTixnOJ-cAu9pQhPaLCkS44QOyUbTl4TnBfZMVHz0uW9l9SRPsit1k91mpkWHbHHNY6KwjgE
yZsdO7W4ryX8HzsVZc6eRjKkm0na90gFbm2nX4FUbbgq1Dn0NgBmnnLSitk4utW5Sws5DW0DvIkOWrDjKWHCgdq3SAju0z18NfM3C-K6CgNs1dAd-EEz_PmM
jmzSbhrzvmRoE5vtpzFM-RPT5dlx-1QyxvEvf_Fq13KsOt4A6DyO2c42VdcyVKdOk4AA010PbqV3ufHXS5t__yWkZti0yW3GcT-7_m
}"#;

/// `aws/Storage/SimpleStorageServiceGeneralAccessPoints`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGENERALACCESSPOINTS: &str = r#"sprite $SimpleStorageServiceGeneralAccessPoints [64x64/16z] {
pPK9kjj028CZDj__nisuLZnV37ZfglRj_f4O9kJrFmiG0PwXwWiL9ZTf3RAKosqwJEKjy-t7YnKGHx-VpVWby_6a7Qkk5SuZE7toN7e1BqsCoJeyS7Icgp11
RH39h-FJaRHde3ck6yG2A70KVrk7ttkmUIsQOe3YAKQBF5MCZpbEPO_yrp8owDI1vZwUftAQjW4idqoCLAkfzK-MitCCSJPLKkHNSvbaKwRCIDPmJcrLT_dK
09Bs2SnsehimdQaSpod8nr3QJjqhNoFWS2k--kDoZ_URF4HQliylmVDFyfEFaZU-Zw3c_JE_iBTyUaJ6C4M3DvGc5-RFwoPy9PRpryz35ERJujZB-uTgO4pB
GZohuW2xsq8-isAeUWLm2R7LI_k7-rk1CKKclR7_vZELOl_r0NY10eqzvrj8JOMr4ljYBzNx-E8x2CpmrMJdOInURIhmBdS5QXx1vfi2WOgNfSSvAirvFyLf
4dhaZJVWJ1W-OPXvCMw4Av9OXfIyBqEErsM93u3yJPebZ7out03J-Vy3NdR-IGtVprylJ-xxhgwySfdVBp1uvU9wKC37V-Bt2snwFdtYlr90_iNzeO3N_wqV
}"#;

/// `aws/Storage/SimpleStorageServiceGlacier`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIER: &str = r#"sprite $SimpleStorageServiceGlacier [64x64/16z] {
xTO5UYHN34NXSqLt_oislb3RAbSucPc_6HvB-ih_EJ_yq9KXtYaYQtnR1vzHplt6aTKpVZNJLMSoERxK0kBpoNOAI5z8K7at2Omt1ULh4-0DjByKrD_LNZVc
BsWl0Y3RcobnjO59240VmstwX422DqLwKhoJuX17b0tUD74Y9SI3CoStycR0lvZfhXVz7iX74Lw1lvAaLsB-sF4VGNz8Y7aSehrAwd537hSBngiYFxH7TNkB
cdYSa03AFWPviVrQufV2qz3UmLhJrAT4_940c3ofbGHe_a00b011ZuW_51qdjIIuZivT0q1wKOGlJNjwl6daFsFWCZc2yQMjVRV13chAg_q00v1T0EDTqDlf
d_RAyNK6BqIPvUSjWu-IxOP4vzFD33w4UaP0pIJqxGm-n2_-G0OFI8BpabvizGtYJABjtZPnhwyTjWiLEfzM1RsbNbla-5e0sKlyDQ0xrs0eigfVL5Ms0y4I
lrkX-KLunZ0EokCxfYipNcLM-qN4E3N-3_dXXsS
}"#;

/// `aws/Storage/SimpleStorageServiceGlacierArchive`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIERARCHIVE: &str = r#"sprite $SimpleStorageServiceGlacierArchive [64x64/16z] {
xPPNOYKn20G10NN_4s-OthosZ9nJVRtG9GGJxRKHE_nEdJrkUuezlTSJAcwdeBQz0xQZOUsRZwtlsqH1sfw0tZLlUnGQDByVmRpv-GXwq_nW10dK3A26v-ty
F5vzGjhZumVSFByv_UZpWrvJwaQ_RCgQ-9--hsgwC-xbUm6YuyHNUW5GfLLIAyR0BoWt1qfnJzrn4dhWf-evyRkLNjfF3NmrhdjaTdT-ZqO37SFvAUs1oDyh
ZdmbH2CqmMdzZDyb0w3Srxn-mJL7rOphhwQdlh85DtuzzxMT0xyULb_PnU_-ywp_1M46f7JS77Bi6xH46c1TxSVShyi90EaERNFVKMBrPPitnDHNGaA1zFdv
AnAek7mlzEVSLmQYm03oVlU_gtxJSR_xh_OVVl_xl-RtSNtpp-V_sFSf2VxOxt_D4VOllau8Uory1G
}"#;

/// `aws/Storage/SimpleStorageServiceGlacierVault`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICEGLACIERVAULT: &str = r#"sprite $SimpleStorageServiceGlacierVault [64x64/16z] {
xTU5RI1H54DHFtJ_5U-64qjMcFEFSE0E8xwPV2W0X8p14uOYaHI3HoLLeQWIZrcoybRTz39cyOWc6tVYfWypWnNamlIwWV7OrkSJVJwo1oM7r_SOik2H3D_h
I4AO9PlURwNRCz_xewIZm_LkzDg3nrPpqqyhkaD8uwPtpALdR22MHkCRUNpxwVIx7pzo-v7p9zRs0o7NZvfVqP_-zAS__UbFV_hJRmhvVr9Bllltsz9999rz
_CTL_ryDV1l_0G
}"#;

/// `aws/Storage/SimpleStorageServiceObject`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICEOBJECT: &str = r#"sprite $SimpleStorageServiceObject [64x64/16z] {
pLTbeeCn26kmtV_6hyliymhFycj2WjIW_6S08i0KLInkRuJ96DblEtYtXBuOfbAXzbBJVhOhoW9GxrD0d9d1M0cqdUlClyGzI2iD0ozdkCVUgB9Gu9GUw2bG
dD6toYDstg0t19HXTGleDgrQb_FSsyRHg7mZMEMwzyCt6TI0oWYm9YZbHc1apBadbXKGYWq788jgb55WiObwTzvsnHFXi-Pufu8oau2-8ebnEhDEfBzmExXt
jwk6gV9zoYQJVFtmvI_vICIVhLzw_VBxXoSfiNzJvoT_VlFt1ok9YV1jSNzQulvCtD-fzoFtViLcCNnmxqhk_VPi_p3Vl-2mpVDkWU1scHL0fNkkV_IBxbRR
tRrVjyUuSDvgiFLsPS09G3hc1yFP_E3ecrzYExzGDxhsQBKbmaqhp5_VDJE_3Ow0KBebmXLb1Y3AVyON
}"#;

/// `aws/Storage/SimpleStorageServiceS3BatchOperations`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3BATCHOPERATIONS: &str = r#"sprite $SimpleStorageServiceS3BatchOperations [64x64/16z] {
rTS5biCm40DG3rdtF_6IbaAlZbnK6Nx8t1uVy_7iSPn8tth7cUpwzB7q0-zjBFK2ldBA5n0JlbZ4YdV_vfft8CWr7m38nUCFSitpJN_o-XUPkcVkw2Cl9FPy
h_5-oRU-0wbwyZvuNNpb8xwvvWD8ykNlvf_FV6DynpSBpzCbN_e2riS3rDOt9zSytrGRy9P0purd1ZnFz5wUzMHx_ZVpjlJPWRHnRxtKyYC7SklvIdVioQjf
FdF1rpuAg3Zs2IIvTlu67g3gcv5hxPSNZrTUwZ-scVSNSaklZnysDtuPkkh9WL37Rt9r_bZrpJMVG95htW1oqNztFRflLl6AwuVvzTEPpAuVPzUlpvPt
}"#;

/// `aws/Storage/SimpleStorageServiceS3Files`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3FILES: &str = r#"sprite $SimpleStorageServiceS3Files [64x64/16z] {
pTOtWaCX48FX3gcR-v_MEG_C3yugMNr-YR3nZx5kuWMe8cmvY3cermibvHfWgZRI-HVQuuWNvyQZZuNqlC1KTTCifJ9yFMpg9c0B_zhVGvE9WT-5SVvyQQIY
0HxoAH-IOAAbCdiTC-azwztt6db15xsKXXluD1FQuoZxVMJh7De1-IZJ-BKaFtYGTgpsV9i_sY8wNekdO_OKe-TqE7jkikUrEhbnDmLxNWkvu1Uz-CZHOrxr
-ho7PtxHLpMh2dlLILwl-hL9mHxc6thphAYxV0DmBg27ymuV0Fv1mnke9noyGPnpV0DmKh2dgCpdlS5ZWFyyMm
}"#;

/// `aws/Storage/SimpleStorageServiceS3GlacierDeepArchive`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3GLACIERDEEPARCHIVE: &str = r#"sprite $SimpleStorageServiceS3GlacierDeepArchive [64x64/16z] {
pLO9RknG21isv_ut_aBC3m4rrdIlkwC7CM2_vl5N8GdL_FxEHFK0ZYk0S9LNSYsEUy1f3UrSXBkP9ipCe-c4tcJRKRXXAdfIyzliC7cfFNiinEYEGHnvTWQn
QdBOBk374O_t8EOGKEKuDk57u_XaFZx07-FFzyqFntoWtqYBYTJdGfvHk0erLXsBILVpJyD5_AQIE6ryjG0i4_HQkEXmynN2f0WFcZMEq_mhfrb93toj4_lk
KJDFb8z5oFcgznvilVYxytqxIjwHBajjlO-5tZC_tWLtMtsIq1qDRL_uctN9GBSla0p8kiWD4gTYoMac07b8rN3Eria2nPUxfUo_9ihCRwCDFMelsy4CYcf3
gnfMIpB-P6t0ynUnLQ0x8kWhfeP0Li98912e4RpRmPeb2XzqyFFf8SGIxt0lTJ1rxoxAmQaI0H3t4W6G1QmdSG5mS07e-Ms5QW3SmUNATDOQ_eiB34zEEHZ7
DIGf80TVO1aaPuByAICK5U8BL7G-jFAHYIF45fXNdraAVVZFEqGMYAnssYoG8UIt3WbPOEGpokqkoJjoGn2nPoWFC5dY6C0R8TdB5fuVExG5O8C_pA9UUQpu
TwX-hdcFErrwM3KwsVEBhG0R_-caHk8-TV5_VtLY5-XNoDRFSQSVTZk_e5ybhxTH2aBv1u5JLEqVzfghx_Bl1hy_fBxz5Vm3
}"#;

/// `aws/Storage/SimpleStorageServiceS3GlacierFlexibleRetrieval`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3GLACIERFLEXIBLERETRIEVAL: &str = r#"sprite $SimpleStorageServiceS3GlacierFlexibleRetrieval [64x64/16z] {
pPU9SeCm34JHjVl_VzngWb9iWoREN7qzOT2jzOJZloA9Ih9fg1u0hm3XAf_O6lXCMEj3WmN2tKmJC_S0Yz07Qty7KNceYY_iMB4smK8ixZtaRVX77N9aeF4c
5fyTHX9JwN7i4AGFuR5Xl5hm2tiEL_lvOxZSxTyOq1zMgrjI7nsUT-4gdR1mHXhJrRmqN9RVLEMNKh7QHGAm3AXNnwJNTXQj19qSqYJZAdxUSJq2WcpM8RDE
tVeBosAaVJxzxJ3lYs_R-xmHic4kyxPfpMF2YRhVFYxDeIItzwKhgC2mg5sxxWKOJA0tzzlpe13KnZPFjmkW8i74-soz6Q3uf6xXug04zO3XMeOoqMDMS8VY
BmaZCPKamsLN1Zt_4IDfJKkQCf9N4xgHz4aG0LBQ0VOkaYyL70W74FAzWdj9xbCAxb6u7EAmvaZgS5A69tSM84n0laOj5j8gU4MPP0ndOgs2L_mjOwl_rjfQ
r7dcJzUl1cYaltxj3j1H1whMtY8-7w2H-6kYdQxyym462V-5IVu34Zi33CvWPu325Lq6s9fx_lEMlIqEEWNhQUvazSqc-_u8LZ_Jm57dbithKfDfcwWP1yyV
oqtQLuUqZASN3Mi-GlRwAVpf_KaoxNZ7ZqUng8ChsHYj5I6HLy-eUQpsDJCPScyG4QsFmTL3gT__XXy
}"#;

/// `aws/Storage/SimpleStorageServiceS3IntelligentTiering`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3INTELLIGENTTIERING: &str = r#"sprite $SimpleStorageServiceS3IntelligentTiering [64x64/16z] {
pLS7ikmW38lx_M_y1y8RVL8qoNQzkYmsmh9Oilmgf46r_RuPg1wPDI73LQx4MjGvqfK5As8ptCsqmSmyG2VEKjYOSyPKxI-B7qLNGvfSsTu-G_JTRO89pvod
jY6ZKTiC-V-0LyLo1p479D9TJx00zMHytkNlDSL7tQtVpDYh0zr9e6DopCqJOvJcKFkRDhUGDlQMQ7up-BRRsCpnjgq1W5GvDHyG9YlXYIUs8p3R5kacA9Wi
f84wOpFlC3DL8VBZDGDRp-_5OxyGKEx5-omiwhdXbT1fqile06Y165iEXBPbnHjqkyS6T9oust4k28OxKedxQGE8I5I3yTMS22XU3Ou9RLEohOZIysOOhqF0
Q78mWoL8Q7Nuri4Sen3fqiXQIq04P2HwKqbM2IafgML3HbJP8mvMBx1uoC5jzKHN4GSJzt9BSOWGIrz0L19IZyR7vNX8_yWTGBCQYASfj0QSE9YJT-rD90y9
M1Lmx6q18PI8F2GWIlUOW8QhvcC2Eixa1mJ48M3qGu43-Ztw6MnR0X2KmrsFJrpKXA8gntPrNzVW6JHadUQumc-52HFox15TA1XmWWGUdqnLLgWQxdGv79X-
KbR-_2QRy-U4QEv3mtHpVd5qtWsgkYs8gYw_Wdy
}"#;

/// `aws/Storage/SimpleStorageServiceS3MultiRegionAccessPoints`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3MULTIREGIONACCESSPOINTS: &str = r#"sprite $SimpleStorageServiceS3MultiRegionAccessPoints [64x64/16z] {
pLTPWkem31klkV-9PokggPwVIUSB_G2jvIYMi_3nrX1H5VaZrtC1ONUJACX0wfsn3tQucPfv71csDVWlMP_b_AQmFNhyi8Kb_JprNNetMTrBuF-TgSnJg20P
tyr8YEzWQyLCjtQEWM5p5b01d3HHmxc4sWhCNqVmTIv-Fpm4862GtpuC2D3PmaE--0E2aiefO2o07DbzFU2dfbD0JWxg4HNWn-deAI1NJkw3NvD993i4n5t-
Ec4l-ThmpG7ed-j7_CQXqJ-d-X5ay0_goBzcXcF_Sl_eCo66_ctNLrpc8VY-kG3WVfO-VRL_s7NqDCjpL6IQ7800n4CLikzlO5Yjw4j1yiqTZFan3b-SRFJJ
c_dyyRP-jFjF9TJ6F-JTcS6Y_i4clWU_XAeYwQXU2tOCoKTVQ44QtwzmSfN02M8nRA8l_PUlD0Kh-bMyUiid4oeckFGwy04eOFhCvmGFkjpd8q62liy7j9ht
7XzGei_y-S8qyoqxllGxXk8nDI-7zlTcIJwRVEfvxdN1qwHxgL6a_rxwvUU3ihfywCz7oTJcyWRUjTaLF1GCWJWzwIpd8zSXqawr61OUiaVCv_VPZm-QCbzB
2BuvFlaWgSzAc_K9WIlCVjpcfd1i_doa-dExQByfEHbDRU6NFkt-CrbM7ETDu-sxlP2f38_V05y
}"#;

/// `aws/Storage/SimpleStorageServiceS3ObjectLambda`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3OBJECTLAMBDA: &str = r#"sprite $SimpleStorageServiceS3ObjectLambda [64x64/16z] {
tLU7akKW38lx___nRJHeI9aBPjeLRGkm0XdPa7Vylp2TV_SXMUNdGrJ5FbthX1s7dtzJzjXWvJa17snFEnXQW62sdHYes4wwZcXoYp-4g0eUT8llhtW3EhO5
MBX1nzRo2DpfOGgMbqtt3IemmIvVawuvg0eb8HlcLu1vDY65VGqurvfyQjBctXaVGYGPUdiFXj0XUhQQ5aBtVIT62dZRV0QgvdnqoEy6e1xJBuqmXVns083T
catvUWOmC79nG33u3GF9tMG4FBP3pEsSAcL5Bvire8-DY_hUH05zDa9_a2EECLSkdIEA86MHX92SSu8aFsxnBVHNcd2w0OuwJVajA3XChp95p8DZt_pqcaZW
qlR58O0aLIVbXd114VLAJyqz8dyGuMxp6h35sIEhqkoFxyy_zOaCGhpY-qu0vuFY3y_jvXb6Vw8gmJyHUXjU6SYQT-XF66_aftJEJf1gU1Pek43Esa3yBjUX
TIP-ck6lVHieMCOqhrvQCeLhLS5wn2py2VKYzDD0U_8bCaDu1ouhs1-5hrmKxfZPAW5UFOKQvk0w0TxNCNxpAkydG2cyP5myqTSLnZl7sO6bOMxZyWK_tC4N
94wE5wBUczTGoX80V5Rlke8ddnDOetgdWLC_83A7EWRE_NldLiz0-lVAxINynjf_1nVSCwz_rEX-X53UWMxw0pFUVUrFKU_xYsTW5sAHDwwR7DiG4LM1-Nyj
VW4
}"#;

/// `aws/Storage/SimpleStorageServiceS3ObjectLambdaAccessPoints`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3OBJECTLAMBDAACCESSPOINTS: &str = r#"sprite $SimpleStorageServiceS3ObjectLambdaAccessPoints [64x64/16z] {
pPS7TkCm34IDGRl_ZJTy6jXOGQ7BradJC_7Hn9RRVoiqVO1xDldxV7G-Fk91VSHxviELlnuLzVJW7_6HyRHe-VghdjdvJDUl1s4hTdY6wD4xkr5sz53b6D_X
xlN6q4TG7w_nsE1yxpf7DRF6OuEllqnSrDp3tS0o3h0xc7X9BAbH3HtiS7624VMWMmV4UyF3K7NmlMmSPEDbcJkjUpLeVFxC2ntmCMXwgaWVUBdlGQDxwf8g
Ibbhd71o7BKvjEp14XZz93quP_IhvkeHmfZ7C3fCudBcLGbUwt4QwJUvSGTY4D1sRXTLH1VJTRuyL08-WpYBcCsBunV3I0KhUOGVYU4KdWIgPsM1p5WKzwGb
TdRP2WxTHm6mzTKx8YYBoBHANvBYQgMOW5UWQ3nw4v2bArxlrXYyL746e5g9kuhIfpmgqIdPy_hRUJYth-VyhV9tCsFzblqiwDPufV_AFPz1z_tdz0tYnlN4
lU5yh6BE7pkjFhwHFf0PO_umspd_Puwq88A-VhWrdUFxOzdnuRH-Mklcph0AEz9j_kRSu4Sp0II5SWS6kzw_z3o4ZkjLbjItsxUQQVaQIOxzMwvF_y9nlNyJ
B2kxmfMUhyyFYQeSxVGKipArkr-_EB_wgOUuc-BHkLQNiv0cuiqWNOk7pVC_D_S7ff3a9HdyrGi8juse-Bozl7m1egHuo6tW-_iRHPwoz87lxe_aD_V7tVrL
EY_Z_hgzFwy_Ou_ASN_UtTzZpIbTF1Omx-_x_nzgJcLwHp3mZT0Epe-a3V_S0MwVIub_B97R_wqV
}"#;

/// `aws/Storage/SimpleStorageServiceS3OnOutposts`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3ONOUTPOSTS: &str = r#"sprite $SimpleStorageServiceS3OnOutposts [64x64/16z] {
xPS5ikCm40NzXlh-9rxm1vKLZPnbx8AFBNcmukEtm9ReIJlei2VrRpe0ocs5Gzxhrj-Iy7Fy__v_txXWFzjNND2_bd_P1Nst_ClSFKxSz8L-51CLzkJpgsW8
tUTFoKc6IQ-_Cekttzxv0YZNuYzU0Dhx1QasvmdLzw_Ew-FOaA0s_tRwPRrktFZ7ElmJHG7iu-UL31_vbtaKdDHj_eAE2iZM7xDZaOzkVeuAOky7l8Lh7Z7r
ly3tleFsGzVupQyZcjCRVsgWXD8Ykdv-8ETofetZwiulQlwGD7gInk-NdsGvPXyexVs0PYMa2m3hVKK2lZYr6BH-m4bfFVklMECdG1OqFUIrlJ-Wx3NSEgHb
RYD2ZGzrYQFKr7RjE-CN1QievltR5jx10sBHsvqlgF6JwNm8dsVBEz-1N9J3fWWRtu3KgcckqwV6htORLJSQK7xxof2WTZGlZCoswsow95f_7R8z_9CPb7sc
cfQljv3b2LYthNpvDkK5QUvHpJEEa99RKhjj6UoeiEODLkFxDPAEtuwF
}"#;

/// `aws/Storage/SimpleStorageServiceS3OneZoneIA`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3ONEZONEIA: &str = r#"sprite $SimpleStorageServiceS3OneZoneIA [64x64/16z] {
pPS7hkqW34HnFVlVyLSKf12BQuMlzkRL5DVnS67yLOY8gCg5GqFro6H5PhZA5rjBVYEizQ7n10ntCxrWvXxv-D2nXtCZJEL7LTompMFh4vmGYxs70D8_TSWJ
YF525fyz3CXIUemJ1FWhV1wPLmiuj-TrTPv_lYuF-rSOyErezKFgRuVpRhfA9woSaTxfwdLfkGo_wLIkN52rHG9fmq1NnpCT2nvq4dHufGcnYfyTrYKWO3SE
3qJKp6-OgOeqxqz_1wZpuiVsNYT23iobJfjEFYgSdFVxrwLPb6HxNzg2LbtuBYsQXXPTb18ANUqA8icSsERjTW1K9BZWVRRU42XUr2rK1rLG5PPh6Se57LN1
7JIV90oob2JBPLy6BWvYWFK4RikOk9lGKT8dGKQ2L0AFNC1_AbZKF3D3_bV1qaZoMnFJcoMXWoGJkbTm3m48yomEsW7OVT8w-oMyXpvH3Fnu2G-GwtTQsG9q
2gv59e1FWpC2Bpk518f18R2nDr27EN5d12PvKqYMXZK4rYRvFJsGfmJEPKRTPTYizvp0ieCxKh22b_4vKt3Eju3g6kpp1Fk6BJuv2D0Xm7vRRjAVZePCxZUX
hDj2oYrw6mx93wvzStxIr_df5jxlLgidlv2kiuNzgAjvLDFrL1RanmEigkxEhph-3lu1
}"#;

/// `aws/Storage/SimpleStorageServiceS3Replication`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3REPLICATION: &str = r#"sprite $SimpleStorageServiceS3Replication [64x64/16z] {
xPO7UkKm28H3pVrllBZDHzOyZRVNIRSz2FmZXRUk_n8rZyZCzr9cHhYfFBHwkMQ52p5RysQKt2CYszL528dJE2KhvkVjaDbkHtpb-VahUmxc8O-quKRPxHkG
AUnhalw2fDJjpwulIUgGVuTZqKXcdCeUusE8Yzr-rQyMy5ko0f8VNtU5Fegin8_u9bz2icA-ILMiktr0DBPRuCzAULkllX2NIR8yhN_q8k5V0LtmEqXIHMyQ
B0iuHFihrEOFaDZag_UlLtz80vZ8Ad46kAcKpXfBQjxRw2D0aDX84eLYITWw2L4i124jaruoRFUjk2LivgVCj3Tq9mtRymO0SjWw2Iu5FjWQ2Iu3lXl08Tbx
DtX_hWCS_8dg1WDiGrwuro-ETNQ0UbiAvzlmmZX0HTCDm9u0r0WxpZV39Jps5A2YZFOVYmBi4dz74iUlHW2EnqOfjcGS0GZ0gOoIA8u0s1f0baGxWc33nY8I
E-qv0lGdkzVDtGX0ldivGBvxEK2oUpb0idivmFNkVGvmthtF0Swx5qE88ZZ3PrFGagAuDJvzFPHmWFu2aLWc0KXRZ0Da5NQ071-_pV7D95V7eBBP3ZFeE5Sx
1uGPoBHaXj47oIrbiPXzdaaDAoARNXCNvZRCSR-XFW4
}"#;

/// `aws/Storage/SimpleStorageServiceS3ReplicationTimeControl`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3REPLICATIONTIMECONTROL: &str = r#"sprite $SimpleStorageServiceS3ReplicationTimeControl [64x64/16z] {
xPQ7Sjum38F5-R__6vVdy0m5ZQcaDxg2ZaG38itFWslNjqJD8pBpLQeV4MugdxHwlXuKBWUpRTwCadj4v3Ox727HHYGBYNalnyC9zdQ-dlFp9tjEPkIHjdvM
sMtzkXJsaQG_a9HQflcHf2uImR7OI6QqzWD3sN-nksQzH-4gd6YJv79dZQ_9gnQbCFaAoJfpYUTBBpiGpRTRzE1AMUf7_SEv1qcMPxj_j4cmVOOEW4OaZuhD
l-wmW3FhAY7stSLeo3xLNwqPoWOmxwpoVWUugPJw6ajgljz6xm7U9AH87XIAa0084g88HCg-jy0VDb3fbDNCl6McAejApZ3FDm0Z9nlIcgN01nj876N0nm0_
I5OMox6F0IvMKb4e3cp8YzPeagwJ02g7eeyFrVa8KEcbHzlE03N2ylNkpQIqxGnGSHduOeq0MUAlEuctN-q4a6NPbQZjZWC6WAB2bz4IVI4KR028SC99j10A
Da5Ms4T8nNe-FmJeiBkfzRU2S8UvsmGmhnbLMPh8YR9z1cXbxsmg746X8ZF03e-hCKg08JF0suWVmS5sCK1WAACg6_WcPu26uOL9sSz0623Yv-L7GDfBJKY7
Z2s4bZ0Q6ZRfL3jMx8WT0xojdfgIsaojWDNln4eUEgEyIVJpsR_fZ8z9tE_DYAxWW0y2m4uRpTw8xsdPiaA5Fg0cCSdfDQXpRySzAFVLlcRvrWEPbipG-Y2v
GHP3x_CvgTqHaOsiBqfAYZxkpzCF
}"#;

/// `aws/Storage/SimpleStorageServiceS3Standard`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3STANDARD: &str = r#"sprite $SimpleStorageServiceS3Standard [64x64/16z] {
pPU9SiCm24JDgV__uuwBCXkQR0RtVZt320HQaCV7VqLELC_V5ntL8tFTanckCl6rM9nqVUMiyCrmDzCJC_D0Ei52s5x3CbMxHV5dthMjoQIynmZnNjs3Cyqp
-zBJPFWuY93t1byhZYj4TqY4cmSumFgYVuxpHy7hu-bVEh-UiU--q86pxiNHcoUszMKCjTkota9wgc-9CJ5J4S4Wgvd7sgIL0v0AX-7ZX4abtF94EM8UT7fD
S63rBscWohYeGHOYFpuFeFlvt1_rTW7bchztON6U3Sk0lUbbrD338h5kLvuG2DnLf5no86qkGf_OYI0TGSSIkRcbCDqF1QExeI49iil0Wle4C104RsN8OqRQ
FlEdaqijXZ1jWW4j8rrE-Gm1Y80qYMegoVCtFLAUH96HE38c8Bz02_dS1E_TKqbB3wTtDC5qUhcHWruZIN5uhyyRW4l56s2wSju0_AwH2_8H0OVoZSs3ZOO2
yawT2yWdXGl8XUBskVmyhxdyymdcDoM_WkVBSADmwjfamY02GGFNDdtScixVFJeIQnDkfb8eNg3mt6DOB4xOI25i1vAKHH6fTpzL5Jd-Bsy
}"#;

/// `aws/Storage/SimpleStorageServiceS3StandardIA`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3STANDARDIA: &str = r#"sprite $SimpleStorageServiceS3StandardIA [64x64/16z] {
pLU7hYGn23kQ_Vz_VDgQ1B4esRdoh3RD1XAC9hz-AkG6bHiEYUe1S0OGhhB3DV0pmbeDZH4mtCtq1ZFtmD3G3sn_1r6fgth9M_7QKe8rnA9M29BmhHLoa8pr
8P9c3oE9L7gqZ29TNyBZY9uP_8BFvUuyVootf-kt1lIojVeczLNm-HIkqXaBpqXhkdgxDKpDRtftxil5QfC5OBU0EWkJFhepeRAWaqkQPCpcPuLv10HPjCF0
FMNp1-nUJFrzqjj5xXS_vdlk23cWI-us9RMUIKZFwy-b6KfIFfUceDKNNgN5Pa6pBw25RZaFCWIlZcs-RXj0HIBPhSpMcmqKJ-uMPe5ig0nRob2Ij--pWpie
HX968fKaosrN1bsS54iJRsk2ZX7pM8HkI_eaY00fHmxE4kIN3YR30OHywM1HgIsv3IMb60a7zCh1H_GiGCOHFGk0sAU3H1BORW01JKbred5m0x77RjdX1kQF
ZpUm3qRwHNvSueEun0_o4jylyesKAtpTta1zyBC2DmL0LLADKVoMcxI3v-cV88FCDLZt2odc8CTdhFhTY_EJpkSd6mArNTJ1Ju3hIBHncvf7fgwdCjinguYe
Qdr-JS_-8tu3
}"#;

/// `aws/Storage/SimpleStorageServiceS3Tables`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3TABLES: &str = r#"sprite $SimpleStorageServiceS3Tables [64x64/16z] {
rTG5SiKm303HmQm6TV_JbbNSUEHHFstxsIySvrqMV6i7Sjuni5NiO4zrTu3lY53FnzLIMya206Bp5Ftwd3jTm3FrR6cGin8zhRvgMFN1ZnmkF9dqYap0ZoUI
tCBkA_2oEjJtux_zCVHv6SzhNOzpK1z3dtrFpBoSxPuFlhKU7ERpG9TWaxLusGy_hkvzJnOpK7VgIIhnxScBryHt5s6-qS1NSsydOC_xFJK9u6AUqrgUHO_Y
Jr-hY_hU2OYXvu9UNo_l-ziV4r-ZfXvszz_M4dk-2hJ6zbSnyDNC-opTuBVh6G
}"#;

/// `aws/Storage/SimpleStorageServiceS3Vectors`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICES3VECTORS: &str = r#"sprite $SimpleStorageServiceS3Vectors [64x64/16z] {
rTQ7RiKW58DXJOurUV-dxMvT2NxagsntD-TBmXJzdb7zpokoC2ss7AMO8z1G4LtFkafK5NKGbqQ3nDMPPa2Six-ljk7zOLHPs_FeCsRlbY1bd-SJJBeprHsy
PpZ4LU2vGPhKiuTGlJRwleDlDEBp_sie10w-nxus-l5IfTvfgUTf1DKuWDmC_mMUtmFw5He0T_2laUUL2PvIZZhx_2NOS-Q-nRvZ1sQGhsYZh_jvNzlWA_6A
FUGuhz7JfiOPVJtJr-zFiMUzdYPhxR1VSmVRjjR-3Kf06-4T-1GgW_5sOnqSfjn4VWKux9aRmK5rvcLZpyCb72XjEQMx_VVAEm
}"#;

/// `aws/Storage/SimpleStorageServiceVPCAccessPoints`
const AWS_AWS_STORAGE_SIMPLESTORAGESERVICEVPCACCESSPOINTS: &str = r#"sprite $SimpleStorageServiceVPCAccessPoints [64x64/16z] {
tTK5ajmm441HDVxxdpXaTwcYYWozuRyuy4H3u-zEz2FO4eZc41PK_de8FR6VewJB2-p0NBab06cFXX3F0zl-kZsscciioLpJlkkzrfdO9NUzFTRTFI3tk3Bm
dT_SekIOlVS9zW7l07ql07ZR1nZGzG9PXzxn2LA_6ruXwa_7ryG1ql0EnDSIyfdNIexfPu-yGYLZMCxytiSN0PMEfOVUv4lMziT1wz_ZnHD2kruvygVUzKjU
Ne2C1Apr-CK15BZnIIxFdnhrQmPsBCUk_10zazgzpOrV-e2nDhs0NpskTVHBw_gzxhIq6tezlo0tVFy629c0pUKyFe0QeBY1RRrkRhJC-b1E6FjotLo4hJFu
cm_NTRUvF_rv-zvxRN2N0BBrNXVt6I0Rxv0o7WpWF_AInQzJ86truXI_Im8WJ4TvzQIMzIHB0Epq2K38U9x4zpvjl4rKJsywUXs-cKwlyfuRKLx9zmCanE4L
iD4R04BhaUyC05cyDG0KxvJ5ksNnRbdyV-qp
}"#;

/// `aws/Storage/Snowball`
const AWS_AWS_STORAGE_SNOWBALL: &str = r#"sprite $Snowball [64x64/16z] {
xTO53W9144JH6gtxdnY7iO9qWaE_6FPPtrv-1ETSYJ28XLg2KQ9IW4o331GAKUHIHgG0r7uR0aaSa0pzkbT0ztqEbDOND8cythTDIsqlzMyASBjlUxFNzfYr
Pq0dqDnhNzL7ID7Q0vgzfs3jlVTUk_Vy_uVhlr2ptN-JQVW1XlaZjFbpx_oxT_xUE__lV_xuViwvBG
}"#;

/// `aws/Storage/SnowballEdge`
const AWS_AWS_STORAGE_SNOWBALLEDGE: &str = r#"sprite $SnowballEdge [64x64/16z] {
xTO7RWCn34JHBuhKtF_6WIiNmHPkUj53IsoFUjCVCKrX55XeNQUeQqs0gy0XjC9mbJYc5J2MVMYIu49IMxPzm7h-4a-H-I1rQJkVNUivIij_0lRpEPfBEMBL
l1gafd9-tQ_A6-kic6TBEQyrtvMV-VJsVBXxwE9D-S75q4LhPzQV9CSD3BhZIgNrVubVc8mBFpio3eQ5f30C_6GUJCdWVFsIUgDrIGVrXruRXacDYoOPDbGu
_-yKGAZJXaQZt_y_k7-Id5lrGzAuDG0_SVyvbaiAyeUr-xTUlpfGx7_z_L4R_rIQ_wBw-alrzL-tll_gSl-Rr-Fhvq_P-ldtfqpJ2m
}"#;

/// `aws/Storage/SnowballSnowballImportExport`
const AWS_AWS_STORAGE_SNOWBALLSNOWBALLIMPORTEXPORT: &str = r#"sprite $SnowballSnowballImportExport [64x64/16z] {
xTO5akGm40NHNo3b_KyyMA5eoR2jPNh3aDLcMz_kl_yYDyLKmxuOUV9Lid87mhJ50d1zw63Qvj3qpZw_oRukMwFakeuzFlzSWmYfCLjwOzQa0AlUvT1pQ1yZ
8ujB2Is73Zxt2Ne0NPC-_TqWvhxDVOFm1m4vpy_RtZcmdLwTHTTMBrzeir_zh3wAIgpqBjFF-rWtr-LfUzude_SyzEDay9lUtaaEASbca5A2t_JZ62MrWlme
yTj-7h3sDQ1OXCbhmzRNom4HhZ7Wtd50MGVufQoBXn_rCS2uvt8kV9TUTYdhUBEZziGMzCfFvCWF--ysZ_0bFnu_DxaWHsvsseU_K-K-2SZAltJ--KSkLlVd
ZuzU7fVdVroUlxAYOYkLOx-XjTZiV_FhTq3Qqn9YxerD6hr1IiwERkDG7myB7a_poiQvsBIjGJpSx5jkQK1NiSwsoalqxJeqiorwzzz_hm
}"#;

/// `aws/Storage/Storage`
const AWS_AWS_STORAGE_STORAGE: &str = r#"sprite $Storage [64x64/16z] {
xTB50OL054FH1Es_ukyQd76vA-myRCQugjKQI3pcCOa7lUw3PfwWaHy96daPSDZhW6sFH-zz6N3Gl-C1FxkUZddDgyUpG5vAq4FoxRKIV8DTDKiUEplsu5eQ
liTV_L4FwRYNCwbvoVJxnxz-gYUwRTw1owTsrItF9ZQZy83LQbS
}"#;

/// `aws/Storage/StorageGateway`
const AWS_AWS_STORAGE_STORAGEGATEWAY: &str = r#"sprite $StorageGateway [64x64/16z] {
xPL50jjG30NdYNp_4nS3-W6vpgRasHb6y77xypWv2HCWpuzivuRgi5q2nNSl7OWFzEobs35VQ0btC82ZzKKTQg6WjfM4DC3gbz8_LsjSo7s_fUy5U6rbKFlb
ipqJSQd3z_n4Kt0TRxy3NdDsHdzd5XmKSti6n7jzjkU_QftV2fS0PEik3jWVkX0BTWanJAF8PVzRP7slez95RpjReWQq3Np9jzIZQijWy8L-s9UyyhoqhVRd
CwBwv-2Eul9-okz4ymN4zyIL3ZyC6FjFgBiFkKnPyqMyMPsgvc-D_ihHsxsBr-nN27BQ7xE_SoEaQ_PbiOtiz_z7-wT_-gV_W-7LZF4T6rxz8PoSV0K
}"#;

/// `aws/Storage/StorageGatewayAmazonFSxFileGateway`
const AWS_AWS_STORAGE_STORAGEGATEWAYAMAZONFSXFILEGATEWAY: &str = r#"sprite $StorageGatewayAmazonFSxFileGateway [64x64/16z] {
xTM5MWD134NXtDXz_oT6Tl8HRDPmkWSR-wctyli2e0gSaUR9BbsFuBRimlPfBPpXfe-PHos5xjHXUBq-hbFgjawRtIpMxgoeIzZu_vZjjfwcGVhSgsn6netw
or_-ykvspdS_uDKg4NXcXXxppWfACadwGQzOaXo7L0zwrA6-O7CTIsxjnqQ7Dzwa0iSYYVRZRvVj2_NJnvPnq6JkwUsNWZ66HPKXRxpsYSSUlTjZOocrhmmf
AnvFVeJw3AGD8a6T-Rul-_7hZt8lapBthn-_3ZckYYJjZ1TR71qaJlkWYZ7E-2IA9ks0rtG5D1a2if-6UpsIIo6ITSoOcnxeeRaRUgq64x-lR_HzzelyvI_l
kf6l-ZtDFVPvlFPGHGrs1FbTtGC
}"#;

/// `aws/Storage/StorageGatewayAmazonS3FileGateway`
const AWS_AWS_STORAGE_STORAGEGATEWAYAMAZONS3FILEGATEWAY: &str = r#"sprite $StorageGatewayAmazonS3FileGateway [64x64/16z] {
xTK5agr038NX-Dd_Yfz0QfeRh3DkVmdwuKAVC6OMONuE5NECmhIp5NNi2vlR15kkxwRoFpMBdANtsjTzIPsVnsGJ-puxK8sRBMWklzc-qab3Ns_Xt3lTvb_O
__fVpo9TN_jGpseU0KIuANUzeMGj7rWA_rVaUCDR-RIA1Q0zlprdrW1itklJuPlb2RJsBpX89dsk8UljMs-W640zlnv8X2mZVkJ5HdhuyWGGCaZnsCRb1d1w
87J6Hxa1ihsGZZJ5-m2RY7ZSBZPaVk4f7Zm1Ld_8Y2lFDbes5xltGIvyLcz04VC-Ry0GrlC2oCEKzZod6BUpvma8PLusnHe0DRtZ8Uzwyi8RFXEr9HMYdgzz
e3VJhg_hVbl_wqFkklRtNVcOyaer4SwHcUXJzHS
}"#;

/// `aws/Storage/StorageGatewayCachedVolume`
const AWS_AWS_STORAGE_STORAGEGATEWAYCACHEDVOLUME: &str = r#"sprite $StorageGatewayCachedVolume [64x64/16z] {
xTC5TW9154JH-hx_7QSB6RUuzKkGEip5mKz9NzEQPxscmB9wLRdXKw-McpvmjVZ1NjsL5pOkCLvJm5fwxBt2UILB7ijXoQi1AJ_pKWec5JQiqSlPyxUwBmjh
qnElz1GiJ0g8U7XL3NfJFVOKh0IbWw-8IFfml_JuMcRfyC3RNh-dMHJNApruBaW3U7GvrskyLkj-ODfzZPuhZZoJPid7DOxMJ_tzTDzzzzrtttttwilaLRvM
vJjzlT8xfiMfxpvjo3UyZUMMNxJqyvP-VOHXcgtlVPdCExdrn_G2
}"#;

/// `aws/Storage/StorageGatewayFileGateway`
const AWS_AWS_STORAGE_STORAGEGATEWAYFILEGATEWAY: &str = r#"sprite $StorageGatewayFileGateway [64x64/16z] {
xSz7WWGX28LX7ydxdtYcSqM1haqd_bK5FvNsdW4W04SaY_L7fapFuCdEKiffkc5JOJh5qrK4IMr2wtdqUWcu6ZQ-AIJSGODBqdNxuUzfUtS76djkKUnw2Ztv
ldpvykNBOrQRWWWoNlkiZOXzdwT7lD4jwvsmd-Tr-ZuE2Tzix3dZ6-ydeVUZj_JbovSlRuXYpsTolEKypJo8CBrm9cflqJy
}"#;

/// `aws/Storage/StorageGatewayNoncachedVolume`
const AWS_AWS_STORAGE_STORAGEGATEWAYNONCACHEDVOLUME: &str = r#"sprite $StorageGatewayNoncachedVolume [64x64/16z] {
xTUr5GD154FHWG_zT-nm69IOLY_LkGKeV4i9QUJcI3PunvfuIxVPr6kuJV_AqzFJqzFJqzFJqyVS5o7lGuRvCT1hh5FSXxh_zkD_clZZVweRVTVnF-PGbxtD
ZlyLM3_Q2m
}"#;

/// `aws/Storage/StorageGatewayTapeGateway`
const AWS_AWS_STORAGE_STORAGEGATEWAYTAPEGATEWAY: &str = r#"sprite $StorageGatewayTapeGateway [64x64/16z] {
xPS55jmm34Gp8xh_YGj-aQDiezJPCipoMjy2q_Fs6meW0Rn3YdbCkGcUm9fiaIlMQ5ewLI59qGp5PO7sOQby-NiFIUzf3D_6jks-1nrqdwUC7jYs9zrBFC9w
lhHr-dFv__n__Z-lAc_m084tO-FSpatmc1UFAiCZ7eDs5Q5ejm7tl0m0vxF5iSfhSLVF5wxnUcbg4O4LdesXHFW2Z-gTei8IbxLyZLEEasz8ZoP-ZSUCKkCG
IrlBfm_abp02aAFqcbzwFh_BX0OMs0FGUvwP_K1IaXOUSS_hZcdsSyvAzi-Mjzs1bqgLi1Z1bgzSX5wP849wlhUh_Vv0ljfP5_zY_QAfdooDdvmD_HXMF9u_
0zEr-RFb4c6PlpR-N9g_cvMr4Yu4DQAiAIwlNuafn-BwhRbkq9sMeuhi-THf_w8G9RKgRtWsExN6YtZ1P-9okN_RLEi_Ep1yUHoYsUCdNvl3UAOTBtjyp-VM
RIhp6kN2AN-PtQxDXQgLBjub6zcsA5nh-stq4G
}"#;

/// `aws/Storage/StorageGatewayVirtualTapeLibrary`
const AWS_AWS_STORAGE_STORAGEGATEWAYVIRTUALTAPELIBRARY: &str = r#"sprite $StorageGatewayVirtualTapeLibrary [64x64/16z] {
xPS7sgCm34H7igIv_uttOzAy50UJxVk_T4LlC1tmf-0pO4DnXi8Qph7Zbv-bTlt4MVAlyKrC-ragHL0WEESt6c3qfTw0QLzM8oxxbK3cTJzAvcNVQ4PSzQMr
-iJFoFZ4zw9zueD4xrTLJFdk46u0p1V-YltloxV8od3qk6fsmhVYZVAN_QpPExzneRWGk15TpSU-aqnjSOlKy8WCrL818ryDGb6oNZL5ZVpGDE-uIaZfhu0O
-CLQhwRrWP5rx9lI-uavl1-IHcX7_hhNoBQ4s2fpvFlOLyl9yQjNRsV73swNdwsN7yVhp_50fWmxk_wChsQhMvYJPVqsDV3Hxfjz_qrRSlTjvAk5bSc54BVl
cQKInZwyU4mvXhveMVjo-Yyw_dtvrjROb5zSKrC-jqpwQJqvxwFdV_3BN-pux-wVLgpz5M_l7njw6h-dtjw_JlpxM_a6
}"#;

/// `aws/Storage/StorageGatewayVolumeGateway`
const AWS_AWS_STORAGE_STORAGEGATEWAYVOLUMEGATEWAY: &str = r#"sprite $StorageGatewayVolumeGateway [64x64/16z] {
xPS7RiGm30JPblp_YwyeH6G8nuXERuDwPTozAz8tXLc4cLzZAZoUSIYVaItT8sxSimKfm5HkgC7p6stO_hGpMRxEGxBDaOzzGuhVTbkmOJDLQDmHgfXsXNe4
Nl11ExxI__V_VS1wViqN-2ngXZjcAbs_ZaDervUZx34nxlZ8tGdFKj0y8DdugHiNGHIoypd2kGv3x7o9KAhmy8OVLXP5nx-JxRVMISzFODbciyvwlWZY4SRU
yZ4E5nvhdKlt-9K6dCoMRlbPnqmJLkkVFmw7VSFdnVEMBoHul1f1qy_N7pu0CW3QVi5R_N__t_yX-Ichp7e-FrNbpldyz28_qV8jutx9p-hwrIjwpcdiSFri
wylOtuF-YHjez3_xdAO61aGKauoxLVFOYXlraMMQD6Kw3m_e6t85
}"#;

/// `aws/Storage/all`
const AWS_AWS_STORAGE_ALL: &str = r#"sprite $Backup [64x64/16z] {
xPO55Xin34I_BBh_YKk1UNLZTSluWsiGpaB_hlpdFxLWLNyj2G3PNudXtOxrLmBw_WNx2w8xeFfh200pWFZArWL0V4qJ0yX7yld58JnTmZEOlapA7oYUlExS
IYsyKNOnXBH3iW6MdosvoxbG27vkHd73MrH6o6SxD-frsjxjh8zJ0JdSrengXpPWkIyWjZLNzgkE3ZxjLsnQ2gYE2Ib3X_YJ6pUt0AXtM7Vm88_z8zCWknU0
vY0JhCfWvNp5IEzFSBmzxt_E60VRYJqyL9-9ayOBavqpfVFDN_fVH4yO-T9FuB8_uBZotErkhH_hj5Wj0cHcG7uBl7Vz9VIC2fGwxm9S4XTp1nDOaUd6LfGH
XLZE3KkT1IDxYIfyBKzDHOivWTX6hpHV4fGhHIvjp1ZLXks3Lf1zWGLWlM40TIr-WFpQHo5TekVHkV-mexaVkt-E_wlgfz6w-kIJmwFiRyD__lCM
}"#;
