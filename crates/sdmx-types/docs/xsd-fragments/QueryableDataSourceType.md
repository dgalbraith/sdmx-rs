<details>
<summary>XSD contract: <code>QueryableDataSourceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="QueryableDataSourceType">
		<xs:annotation>
			<xs:documentation>QueryableDataSourceType describes a data source which is accepts a standard SDMX Query message and responds appropriately.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="DataURL" type="xs:anyURI">
				<xs:annotation>
					<xs:documentation>DataURL contains the URL of the data source.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="WSDLURL" type="xs:anyURI" minOccurs="0">
				<xs:annotation>
					<xs:documentation>WSDLURL provides the location of a WSDL instance on the internet which describes the queryable data source.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="WADLURL" type="xs:anyURI" minOccurs="0">
				<xs:annotation>
					<xs:documentation>WADLURL provides the location of a WADL instance on the internet which describes the REST protocol of the queryable data source.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:sequence>
		<xs:attribute name="isRESTDatasource" type="xs:boolean" use="required">
			<xs:annotation>
				<xs:documentation>The isRESTDatasource attribute indicates, if true, that the queryable data source is accessible via the REST protocol.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="isWebServiceDatasource" type="xs:boolean" use="required">
			<xs:annotation>
				<xs:documentation>The isWebServiceDatasource attribute indicates, if true, that the queryable data source is accessible via Web Services protocols.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
